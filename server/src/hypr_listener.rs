use regex::Regex;
use std::{
    env,
    ops::{Add, Deref},
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::Mutex,
};

use crate::common::{
    HyprEvent, HyprvisorListener, Subscribers, SubscriptionID, WindowInfo, WorkspaceInfo,
};

pub struct HyprListener {
    listen_sock: Option<String>,
    command_sock: Option<String>,
}

impl HyprListener {
    pub fn new() -> Self {
        HyprListener {
            listen_sock: None,
            command_sock: None,
        }
    }

    pub fn get_hyprland_command_socket() -> Option<String> {
        match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(value) => Some(format!("/tmp/hypr/{}/.socket.sock", value)),
            Err(_) => {
                eprintln!("HYPRLAND_INSTANCE_SIGNATURE not set! (is hyprland running?)");
                None
            }
        }
    }

    pub fn get_hyprland_event_socket() -> Option<String> {
        match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(value) => Some(format!("/tmp/hypr/{}/.socket2.sock", value)),
            Err(_) => {
                eprintln!("HYPRLAND_INSTANCE_SIGNATURE not set! (is hyprland running?)");
                None
            }
        }
    }
}

impl HyprvisorListener for HyprListener {
    fn prepare_listener(&mut self) {
        self.listen_sock = HyprListener::get_hyprland_event_socket();
        self.command_sock = HyprListener::get_hyprland_command_socket();
    }

    async fn start_listener(&mut self, subscribers: Arc<Mutex<Subscribers>>) {
        if self.listen_sock.is_none() || self.command_sock.is_none() {
            eprintln!("Failed to get hyprland socket");
            return;
        }

        let listen_socket = match HyprListener::get_hyprland_event_socket() {
            Some(value) => value,
            None => return,
        };

        let command_socket = self.command_sock.as_ref().unwrap().to_string();

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
                    if let Some(HyprEvent::WorkspaceChanged(active_id)) = events
                        .iter()
                        .find(|event| matches!(event, HyprEvent::WorkspaceChanged(_)))
                    {
                        // Call do_sth with the data associated with the first matching Workspace variant
                        let subscribers_ref = Arc::clone(&subscribers);
                        tokio::spawn(broadcast_workspace_info(
                            Arc::new(command_socket.to_string()),
                            Arc::new(active_id.to_string()),
                            subscribers_ref,
                        ));
                        // broadcast_workspace_info(command_socket, active_id, subscribers_ref).await;
                    }

                    if let Some(HyprEvent::WindowChanged(window_data)) = events
                        .iter()
                        .find(|event| matches!(event, HyprEvent::WindowChanged(_)))
                    {
                        let mut window_info: WindowInfo = WindowInfo::new();
                        window_info.class = window_data.0.clone();
                        window_info.title = window_data.1.clone();

                        let subscribers_ref = Arc::clone(&subscribers);
                        tokio::spawn(broadcast_window_info(
                            Arc::new(window_info),
                            subscribers_ref,
                        ));
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

fn parse_event(data: &str) -> Option<HyprEvent> {
    let mut parts = data.splitn(2, ">>");

    match (parts.next(), parts.next()) {
        (Some("activewindow"), Some(window_info)) => {
            let (class, title) = window_info.split_once(',').unwrap_or(("", ""));
            Some(HyprEvent::WindowChanged((
                class.trim().to_string(),
                title.trim().to_string(),
            )))
        }
        (Some("workspace"), Some(value)) => Some(HyprEvent::WorkspaceChanged(value.to_string())),
        (Some("activewindowv2"), Some(value)) => Some(HyprEvent::Window2Changed(value.to_string())),
        (Some("createworkspace"), Some(value)) => {
            Some(HyprEvent::WorkspaceCreated(value.to_string()))
        }
        (Some("destroyworkspace"), Some(value)) => {
            Some(HyprEvent::WorkspaceDestroyed(value.to_string()))
        }
        _ => None,
    }
}

fn process_event(buffer: &[u8]) -> Vec<HyprEvent> {
    String::from_utf8_lossy(buffer)
        .lines()
        .filter_map(parse_event)
        .collect()
}

async fn broadcast_window_info(window_info: Arc<WindowInfo>, subscribers: Arc<Mutex<Subscribers>>) {
    let win_info = window_info.deref();
    let win_msg: String = match serde_json::to_string(win_info) {
        Ok(msg) => msg,
        Err(e) => {
            println!("Failed to make window info JSON. Error: {e}");
            return;
        }
    };

    let mut disconnected_pid = Vec::new();
    let mut subscribers = subscribers.lock().await;
    if let Some(win_subscribers) = subscribers.get_mut(&SubscriptionID::Window) {
        for (pid, stream) in win_subscribers.iter_mut() {
            if let Err(_) = stream.write_all(win_msg.as_bytes()).await {
                println!("Client {} is dead. Remove later", pid);
                disconnected_pid.push(*pid);
            }
        }

        for pid in disconnected_pid {
            win_subscribers.remove(&pid);
        }
    }
}

pub async fn broadcast_workspace_info(
    cmd_sock_rc: Arc<String>,
    active_id_rc: Arc<String>,
    subscribers: Arc<Mutex<Subscribers>>,
) {
    let cmd_sock = cmd_sock_rc.deref();
    let active_id = active_id_rc.deref();

    let mut command_stream = match UnixStream::connect(cmd_sock).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!(
                "Failed to connect to command socket: {}. Error: {}",
                cmd_sock, e
            );
            return;
        }
    };

    if let Err(e) = command_stream.write_all(b"workspaces").await {
        eprintln!("Cannot get workspaces info. Error: {e}");
        return;
    }

    let mut buffer = [0; 8192];
    let mut line = String::new();
    let mut ws_info: Vec<WorkspaceInfo> = Vec::new();

    loop {
        match command_stream.read(&mut buffer).await {
            Ok(size) if size > 0 => {
                let chunk_str = String::from_utf8_lossy(&buffer[..size]);
                for ch in chunk_str.chars() {
                    match ch {
                        '\n' => {
                            if let Some(ws) = extract_ws_info(&line.trim()) {
                                ws_info.push(ws);
                            }
                            line.clear();
                        }
                        _ => line.push(ch),
                    }
                }
            }
            Ok(_) | Err(_) => break,
        }
    }

    // Sort workspace info numerically based on id
    ws_info.sort_by(|a, b| a.id.cmp(&b.id));

    if let Some(item) = ws_info
        .iter_mut()
        .find(|info| info.id == active_id.parse::<u32>().unwrap_or(1))
    {
        item.active = true;
    }

    let ws_msg = match serde_json::to_string(&ws_info) {
        Ok(msg) => msg,
        Err(e) => {
            println!("Failed to make workspace JSON. Error: {e}");
            return;
        }
    };

    let mut disconnected_pid = Vec::new();
    let mut subscribers = subscribers.lock().await;
    if let Some(ws_subscriber) = subscribers.get_mut(&SubscriptionID::Workspace) {
        for (pid, stream) in ws_subscriber.iter_mut() {
            if let Err(_) = stream.write_all(ws_msg.as_bytes()).await {
                println!("Client {} is dead. Remove later", pid);
                disconnected_pid.push(*pid);
            }
        }

        for pid in disconnected_pid {
            ws_subscriber.remove(&pid);
        }
    }
}

fn extract_ws_info(data: &str) -> Option<WorkspaceInfo> {
    let rgx = Regex::new(r"workspace ID (\d+) \(([^)]+)\) on monitor ([^:]+):").unwrap();

    rgx.captures(data).map(|captures| {
        let mut ws = WorkspaceInfo::new();
        ws.id = captures[1].parse::<u32>().unwrap_or(1);
        ws.name = captures[2].to_string();
        ws.monitor = captures[3].to_string();
        ws.active = false;
        ws
    })
}
