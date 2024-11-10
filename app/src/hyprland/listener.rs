use super::{
    types::{HyprEvent, HyprEventList, HyprSocketType, HyprWinInfo, HyprWorkspaceInfo},
    utils::hyprland_socket,
    window, workspaces,
};
use crate::{common_types::Subscriber, error::HyprvisorResult, ipc::*};

use std::sync::Arc;
use tokio::{io::AsyncReadExt, net::UnixStream, sync::Mutex};

pub async fn start_hyprland_listener(subscribers: Arc<Mutex<Subscriber>>) -> HyprvisorResult<()> {
    let event_socket = hyprland_socket(&HyprSocketType::Event);
    let mut current_win_info = HyprWinInfo::default();
    let mut current_ws_info: Vec<HyprWorkspaceInfo> = Vec::new();

    log::info!("Start Hyprland event listener");
    let mut stream = match connect_to_socket(&event_socket, 1, 100).await {
        Ok(stream) => stream,
        Err(e) => {
            log::error!("{e}");
            std::process::exit(1);
        }
    };

    let mut buffer = vec![0; 8192];
    loop {
        match fetch_hyprland_event(&mut stream, &mut buffer).await {
            events if events.contains(&HyprEvent::WindowChanged) => {
                send_window_info(&mut current_win_info, subscribers.clone()).await;
                send_workspace_info(&mut current_ws_info, subscribers.clone()).await;
            }
            events
                if events.contains_at_least(&[
                    &HyprEvent::WorkspaceCreated,
                    &HyprEvent::WorkspaceDestroyed,
                ]) =>
            {
                send_workspace_info(&mut current_ws_info, subscribers.clone()).await;
            }
            _ => {}
        }
    }
}

async fn fetch_hyprland_event(stream: &mut UnixStream, buffer: &mut [u8]) -> HyprEventList {
    match stream.read(buffer).await {
        Ok(bytes) if bytes > 0 => buffer[..bytes].into(),
        Ok(_) | Err(_) => {
            log::error!("Connection closed from Hyprland event socket");
            std::process::exit(1);
        }
    }
}

async fn send_window_info(current_info: &mut HyprWinInfo, subscribers: Arc<Mutex<Subscriber>>) {
    if let Err(e) = window::broadcast_info(current_info, subscribers.clone()).await {
        log::debug!("Window: {e}");
    }
}

async fn send_workspace_info(
    current_info: &mut Vec<HyprWorkspaceInfo>,
    subscribers: Arc<Mutex<Subscriber>>,
) {
    if let Err(e) = workspaces::broadcast_info(current_info, subscribers.clone()).await {
        log::debug!("Workspace: {e}");
    }
}
