use super::{types::*, utils::*, window, workspaces};
use crate::{
    global::{BUFFER_SIZE, SUBSCRIBERS},
    ipc::*,
    types::Subscriber,
};

use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn start_hyprland_listener() {
    let event_socket = hyprland_socket(&HyprSocketType::Event);
    let mut current_win_info = HyprWinInfo::default();
    let mut current_ws_info: Vec<HyprWorkspaceInfo> = Vec::new();

    log::info!("Start Hyprland event listener");

    let subscribers = SUBSCRIBERS.clone();

    let mut stream = match connect_to_socket(&event_socket, 1, 100).await {
        Ok(stream) => stream,
        Err(e) => {
            panic!("Error: {e}");
        }
    };

    let mut buffer = vec![0; *BUFFER_SIZE];
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
