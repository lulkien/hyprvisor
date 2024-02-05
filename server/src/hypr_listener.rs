use regex::Regex;
use std::{collections::HashMap, env, ops::Deref, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::Mutex,
};

use crate::common::{
    HyprEvent, HyprvisorListener, Subscribers, SubscriptionID, WindowInfo, WorkspaceInfo,
};

pub struct HyprListener {
    current_window: WindowInfo,
    current_workspace: Vec<WorkspaceInfo>,
}

impl HyprListener {
    pub fn new() -> Self {
        HyprListener {
            current_window: WindowInfo::new(),
            current_workspace: Vec::new(),
        }
    }

    fn get_hyprland_command_socket() -> Option<String> {
        match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(value) => Some(format!("/tmp/hypr/{}/.socket.sock", value)),
            Err(_) => {
                println!("HYPRLAND_INSTANCE_SIGNATURE not set! (is hyprland running?)");
                None
            }
        }
    }

    fn get_hyprland_event_socket() -> Option<String> {
        match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(value) => Some(format!("/tmp/hypr/{}/.socket2.sock", value)),
            Err(_) => {
                println!("HYPRLAND_INSTANCE_SIGNATURE not set! (is hyprland running?)");
                None
            }
        }
    }
}

impl HyprvisorListener for HyprListener {
    fn prepare_listener(&mut self) {}

    async fn start_listener(&mut self, subscribers: Arc<Mutex<Subscribers>>) {
        let listen_socket = match HyprListener::get_hyprland_event_socket() {
            Some(value) => value,
            None => return,
        };

        let mut stream = match UnixStream::connect(&listen_socket).await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Failed to connect to {}. Error: {}", listen_socket, e);
                return;
            }
        };

        println!("Start Hyprland Event listener");

        let mut buffer: [u8; 8192] = [0; 8192];
        loop {
            match stream.read(&mut buffer).await {
                Ok(size) if size > 0 => {
                    let events = process_event(&buffer[..size]);
                    if events.contains(&HyprEvent::WindowChanged) {
                        let window_info = get_active_window().await;
                        if self.current_window != window_info {
                            self.current_window = window_info.clone();
                            let win_subscriber_arc = Arc::clone(&subscribers);
                            let win_info_arc: Arc<WindowInfo> = Arc::new(window_info);
                            tokio::spawn(broadcast_window_info(win_info_arc, win_subscriber_arc));
                        }

                        // Window change? Just update the workspace too
                        let ws_info = get_current_workspaces().await;
                        if self.current_workspace != ws_info {
                            self.current_workspace = ws_info.clone();
                            let ws_subscriber_arc = Arc::clone(&subscribers);
                            let ws_info_arc: Arc<Vec<WorkspaceInfo>> = Arc::new(ws_info);
                            tokio::spawn(broadcast_workspaces_info(ws_info_arc, ws_subscriber_arc));
                        }
                    } else if events.contains(&HyprEvent::WorkspaceCreated)
                        || events.contains(&HyprEvent::WorkspaceDestroyed)
                    {
                        let ws_info = get_current_workspaces().await;
                        if self.current_workspace != ws_info {
                            self.current_workspace = ws_info.clone();
                            let ws_subscriber_arc = Arc::clone(&subscribers);
                            let ws_info_arc: Arc<Vec<WorkspaceInfo>> = Arc::new(ws_info);
                            tokio::spawn(broadcast_workspaces_info(ws_info_arc, ws_subscriber_arc));
                        }
                    }
                }
                Ok(_) | Err(_) => {
                    eprintln!("Connection close from socket");
                    break;
                }
            }
        }
    }
}

fn process_event(buffer: &[u8]) -> Vec<HyprEvent> {
    let mut evt_list: Vec<HyprEvent> = String::from_utf8_lossy(buffer)
        .lines()
        .map(|line| {
            let mut parts = line.splitn(2, ">>");
            let event = match parts.next() {
                Some(evt) => evt,
                _ => "",
            };

            match event {
                "activewindow" => HyprEvent::WindowChanged,
                "workspace" => HyprEvent::WorkspaceChanged,
                "activewindowv2" => HyprEvent::Window2Changed,
                "createworkspace" => HyprEvent::WorkspaceCreated,
                "destroyworkspace" => HyprEvent::WorkspaceDestroyed,
                _ => HyprEvent::InvalidEvent,
            }
        })
        .collect();
    evt_list.dedup();
    evt_list
}

async fn broadcast_window_info(
    window_info_arc: Arc<WindowInfo>,
    subscribers_arc: Arc<Mutex<Subscribers>>,
) {
    let window_info = window_info_arc.deref();
    let window_json = serde_json::to_string(window_info).unwrap();

    let mut disconnected_pid = Vec::new();
    let mut subscribers = subscribers_arc.lock().await;
    if let Some(win_subscribers) = subscribers.get_mut(&SubscriptionID::Window) {
        for (pid, stream) in win_subscribers.iter_mut() {
            if let Err(_) = stream.write_all(window_json.as_bytes()).await {
                println!("Client {} is disconnected. Remove later", pid);
                disconnected_pid.push(*pid);
            }
        }

        for pid in disconnected_pid {
            win_subscribers.remove(&pid);
        }
    }
}

pub async fn get_active_window() -> WindowInfo {
    // TODO: Improve in the future
    let mut window_info = WindowInfo::new();

    let cmd_sock = match HyprListener::get_hyprland_command_socket() {
        Some(value) => value,
        None => return window_info,
    };

    let mut stream = match UnixStream::connect(&cmd_sock).await {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!("Error: {err}");
            return window_info;
        }
    };

    if let Err(e) = stream.write_all(b"activewindow").await {
        eprintln!("Failed to get activewindow. Error: {e}");
        return window_info;
    }

    let mut response: String = String::new();
    let mut buffer: [u8; 8192] = [0; 8192];
    loop {
        match stream.read(&mut buffer).await {
            Ok(size) if size > 0 => {
                response = String::from_utf8_lossy(&buffer[..size]).to_string();
            }
            Ok(_) | Err(_) => break,
        }
    }

    window_info = parse_window_data(&response);
    window_info
}

fn parse_window_data(raw_data: &str) -> WindowInfo {
    let mut window_info = WindowInfo::new();

    let processed_data: Vec<String> = raw_data
        .split("\n")
        .map(|s| s.trim().to_string())
        .filter(|s| s.starts_with("class: ") || s.starts_with("title: "))
        .collect();

    for d in processed_data {
        if d.starts_with("class: ") {
            window_info.class = d.strip_prefix("class: ").unwrap().to_string();
        } else if d.starts_with("title: ") {
            window_info.title = d.strip_prefix("title: ").unwrap().to_string();
        } else {
            // do nothing
        }
    }

    window_info
}

async fn broadcast_workspaces_info(
    workspace_info_arc: Arc<Vec<WorkspaceInfo>>,
    subscribers_arc: Arc<Mutex<Subscribers>>,
) {
    let workspaces_info = workspace_info_arc.deref();
    let workspaces_json = serde_json::to_string(&workspaces_info).unwrap();

    let mut disconnected_pid = Vec::new();
    let mut subscribers = subscribers_arc.lock().await;
    if let Some(ws_subscribers) = subscribers.get_mut(&SubscriptionID::Workspace) {
        for (pid, stream) in ws_subscribers.iter_mut() {
            if let Err(_) = stream.write_all(workspaces_json.as_bytes()).await {
                println!("Client {} is disconnected. Remove later", pid);
                disconnected_pid.push(*pid);
            }
        }

        for pid in disconnected_pid {
            ws_subscribers.remove(&pid);
        }
    }
}

pub async fn get_current_workspaces() -> Vec<WorkspaceInfo> {
    // TODO: Improve how we get the command socket path
    let mut ws_list: Vec<WorkspaceInfo> = Vec::new();
    let cmd_sock = match HyprListener::get_hyprland_command_socket() {
        Some(value) => Arc::new(value),
        None => return ws_list,
    };
    let active_workspace: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let all_workspaces: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let all_workspaces_ref = Arc::clone(&all_workspaces);
    let cmd_sock_workspaces_ref = Arc::clone(&cmd_sock);
    let workspaces_future = tokio::spawn(async move {
        let mut stream = match UnixStream::connect(cmd_sock_workspaces_ref.deref()).await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Error: {e}");
                return;
            }
        };

        if let Err(e) = stream.write_all(b"workspaces").await {
            eprintln!("Failed to get workspaces. Error: {e}");
            return;
        }

        let mut response: String = String::new();
        let mut buffer: [u8; 8192] = [0; 8192];
        loop {
            match stream.read(&mut buffer).await {
                Ok(size) if size > 0 => {
                    response = String::from_utf8_lossy(&buffer[..size]).to_string();
                }
                Ok(_) | Err(_) => break,
            }
        }

        all_workspaces_ref.lock().await.replace(response);
    });

    let active_workspace_ref = Arc::clone(&active_workspace);
    let cmd_sock_avtive_ref = Arc::clone(&cmd_sock);
    let active_future = tokio::spawn(async move {
        let mut stream = match UnixStream::connect(cmd_sock_avtive_ref.deref()).await {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("Error: {err}");
                return;
            }
        };

        if let Err(e) = stream.write_all(b"activeworkspace").await {
            eprintln!("Failed to get activeworkspace. Error: {e}");
            return;
        }

        let mut response: String = String::new();
        let mut buffer: [u8; 8192] = [0; 8192];
        loop {
            match stream.read(&mut buffer).await {
                Ok(size) if size > 0 => {
                    response = String::from_utf8_lossy(&buffer[..size]).to_string();
                }
                Ok(_) | Err(_) => break,
            }
        }

        active_workspace_ref.lock().await.replace(response);
    });

    // Wait for both threads to complete
    let _ = tokio::try_join!(workspaces_future, active_future);

    // Parse data
    let mut workspace_table: HashMap<u32, WorkspaceInfo> = HashMap::new();
    let workspace_string: Vec<String> = all_workspaces
        .lock()
        .await
        .take()
        .unwrap()
        .split("\n\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    for ws in workspace_string {
        let (workspace_id, workspace) = parse_workspace_data(&ws);
        workspace_table.insert(workspace_id, workspace);
    }

    let active_string: String = active_workspace.lock().await.take().unwrap();
    let (active_id, _) = parse_workspace_data(&active_string);
    if let Some(active_workspace_info) = workspace_table.get_mut(&active_id) {
        active_workspace_info.active = true;
    }

    for (_, ws_data) in workspace_table.into_iter() {
        ws_list.push(ws_data);
    }
    ws_list.sort_by(|a, b| a.id.cmp(&b.id));

    ws_list
}

fn parse_workspace_data(raw_data: &str) -> (u32, WorkspaceInfo) {
    let mut ws = WorkspaceInfo::new();
    let mut ws_id: u32 = 0;
    let workspace_regex =
        Regex::new(r"workspace ID (\d+) \(([^)]+)\) on monitor ([^:]+):").unwrap();
    let window_regex = Regex::new(r"windows: (\d+)").unwrap();
    let lines: Vec<String> = raw_data.split("\n").map(|s| s.trim().to_string()).collect();

    for line in lines {
        if workspace_regex.is_match(&line) {
            workspace_regex.captures(&line).map(|capture| {
                ws.id = capture[1].parse::<u32>().unwrap_or(1);
                ws_id = ws.id.clone();
                ws.name = capture[2].to_string();
            });
        } else if window_regex.is_match(&line) {
            window_regex.captures(&line).map(|capture| {
                ws.occupied = capture[1].parse::<u32>().unwrap_or(0) > 0;
            });
        }
    }

    (ws_id, ws)
}
