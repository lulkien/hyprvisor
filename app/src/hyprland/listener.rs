use super::{types::*, utils::*, window, workspaces};
use crate::{error::HyprvisorResult, global::BUFFER_SIZE, ipc::*};

pub async fn start_hyprland_listener() -> HyprvisorResult<()> {
    let event_socket = hyprland_socket(&HyprSocketType::Event);
    let mut current_win_info = HyprWindowInfo::default();
    let mut current_ws_info: Vec<HyprWorkspaceInfo> = Vec::new();

    log::info!("Start Hyprland event listener");

    let mut stream = connect_to_socket(&event_socket, 1, 100).await?;

    let mut buffer = vec![0; *BUFFER_SIZE];
    loop {
        match fetch_hyprland_event(&mut stream, &mut buffer).await {
            events if events.contains(&HyprEvent::WindowChanged) => {
                send_window_info(&mut current_win_info).await;
                send_workspace_info(&mut current_ws_info).await;
            }
            events
                if events.contains_at_least(&[
                    &HyprEvent::WorkspaceCreated,
                    &HyprEvent::WorkspaceDestroyed,
                ]) =>
            {
                send_workspace_info(&mut current_ws_info).await;
            }
            _ => {}
        }
    }
}

async fn send_window_info(current_info: &mut HyprWindowInfo) {
    if let Err(e) = window::broadcast_info(current_info).await {
        log::debug!("Window: {e}");
    }
}

async fn send_workspace_info(current_info: &mut Vec<HyprWorkspaceInfo>) {
    if let Err(e) = workspaces::broadcast_info(current_info).await {
        log::debug!("Workspace: {e}");
    }
}
