use crate::{
    client,
    common_types::{
        HResult, HyprEvent, HyprSocketType, HyprWinInfo, HyprWorkspaceInfo, HyprvisorError,
        Subscriber, SubscriptionID,
    },
    opts::CommandOpts,
    utils,
};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

pub async fn start_hyprland_listener(subscribers: Arc<Mutex<Subscriber>>) {
    let event_socket = utils::get_hyprland_socket(&HyprSocketType::Event);
    let mut current_win_info = HyprWinInfo::default();
    let mut current_ws_info: Vec<HyprWorkspaceInfo> = Vec::new();

    log::info!("Start Hyprland event listener");
    let mut event_listener = utils::try_connect(&event_socket, 1, 500).await.unwrap();
    let mut buffer: [u8; 8192] = [0; 8192];

    loop {
        match event_listener.read(&mut buffer).await {
            Ok(bytes) if bytes > 0 => {
                let events = parse_events(&buffer[..bytes]);
                log::info!("{:?}", events);
                if events.contains(&HyprEvent::WindowChanged) {
                    let _ = broadcast_window_info(&mut current_win_info, subscribers.clone()).await;
                    let _ =
                        broadcast_workspace_data(&mut current_ws_info, subscribers.clone()).await;
                } else if events.contains(&HyprEvent::WorkspaceCreated)
                    || events.contains(&HyprEvent::WorkspaceDestroyed)
                {
                    let _ =
                        broadcast_workspace_data(&mut current_ws_info, subscribers.clone()).await;
                }
            }

            Ok(_) | Err(_) => {
                log::error!("Connection closed from Hyprland event socket");
                client::send_server_command(&utils::get_socket_path(), &CommandOpts::Kill, 1).await;
            }
        }
    }
}

fn parse_events(buffer: &[u8]) -> Vec<HyprEvent> {
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

async fn get_hypr_active_window() -> HResult<HyprWinInfo> {
    let mut win_info = HyprWinInfo::default();
    let cmd_sock = utils::get_hyprland_socket(&HyprSocketType::Command);
    let raw_response = utils::write_to_socket(&cmd_sock, "activewindow", 1, 250).await?;

    let processed_data: Vec<String> = raw_response
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| s.starts_with("class: ") || s.starts_with("title: "))
        .collect();

    for d in processed_data {
        if d.starts_with("class: ") {
            win_info.class = d.strip_prefix("class: ").unwrap().to_string();
        } else if d.starts_with("title: ") {
            win_info.title = d.strip_prefix("title: ").unwrap().to_string();
        } else {
            // do nothing
        }
    }

    Ok(win_info)
}

async fn broadcast_window_info(
    current_win_info: &mut HyprWinInfo,
    subscribers: Arc<Mutex<Subscriber>>,
) -> HResult<()> {
    let mut subscribers = subscribers.lock().await;
    let win_subscribers = match subscribers.get_mut(&SubscriptionID::Window) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            log::info!("No subscribers");
            return Err(HyprvisorError::NoSubscribers);
        }
    };

    let new_win_info = get_hypr_active_window().await?;
    if *current_win_info == new_win_info {
        return Err(HyprvisorError::FalseAlarm);
    }

    *current_win_info = new_win_info.clone();

    let mut disconnected_pid = Vec::new();
    let window_json = serde_json::to_string(current_win_info)?;

    for (pid, stream) in win_subscribers.iter_mut() {
        if stream.write_all(window_json.as_bytes()).await.is_err() {
            println!("Client {} is disconnected. Remove later", pid);
            disconnected_pid.push(*pid);
        }
    }

    for pid in disconnected_pid {
        win_subscribers.remove(&pid);
    }

    Ok(())
}

async fn get_hypr_workspace_info() -> HResult<Vec<HyprWorkspaceInfo>> {
    let cmd_sock = utils::get_hyprland_socket(&HyprSocketType::Command);
    let handle_active_ws = utils::write_to_socket(&cmd_sock, "activeworkspace", 1, 250);
    let handle_all_ws = utils::write_to_socket(&cmd_sock, "workspaces", 1, 250);

    tokio::try_join!(handle_active_ws, handle_all_ws)?;

    Ok(vec![])
}

async fn broadcast_workspace_data(
    current_ws_info: &mut [HyprWorkspaceInfo],
    subscribers: Arc<Mutex<Subscriber>>,
) -> HResult<()> {
    let mut subscribers = subscribers.lock().await;
    let ws_subscribers = match subscribers.get_mut(&SubscriptionID::Window) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            log::info!("No subscribers");
            return Err(HyprvisorError::NoSubscribers);
        }
    };

    let new_ws_info = get_hypr_workspace_info().await?;
    if *current_ws_info == new_ws_info {
        return Err(HyprvisorError::FalseAlarm);
    }

    log::debug!("Hello");
    Ok(())
}
