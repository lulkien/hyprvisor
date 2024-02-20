use crate::{
    client,
    common_types::{HyprEvent, Subscriber},
    opts::ServerCommand,
    utils,
};
use std::sync::Arc;
use tokio::{io::AsyncReadExt, sync::Mutex};

pub async fn start_hyprland_listener(subscribers: Arc<Mutex<Subscriber>>) {
    let event_socket = match utils::get_hyprland_event_socket() {
        Some(sock) => sock,
        None => return,
    };

    let mut event_listener = match utils::try_connect(&event_socket, 1, 500).await {
        Some(stream) => stream,
        None => return,
    };

    log::info!("Start Hyprland event listener");

    let mut buffer: [u8; 8192] = [0; 8192];
    loop {
        match event_listener.read(&mut buffer).await {
            Ok(bytes_received) if bytes_received > 0 => {
                let events = process_event(&buffer[..bytes_received]);
                if events.contains(&HyprEvent::WindowChanged) {
                    let subscribers_ref = Arc::clone(&subscribers);
                    handle_window_changed(subscribers_ref).await;
                } else if events.contains(&HyprEvent::WorkspaceCreated)
                    || events.contains(&HyprEvent::WorkspaceDestroyed)
                {
                    let subscribers_ref = Arc::clone(&subscribers);
                    handle_workspace_changed(subscribers_ref).await;
                }
            }
            Ok(_) | Err(_) => {
                log::error!("Connection closed from Hyprland event socket");
                client::send_server_command(&utils::get_socket_path(), &ServerCommand::Kill, 1)
                    .await;
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

async fn handle_window_changed(_subscribers: Arc<Mutex<Subscriber>>) {
    log::info!("WindowChanged");
}

async fn handle_workspace_changed(_subscribers: Arc<Mutex<Subscriber>>) {
    log::info!("WorkspaceChanged");
}
