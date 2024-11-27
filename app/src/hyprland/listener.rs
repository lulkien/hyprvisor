use super::{types::*, utils::*, window, workspaces};
use crate::{error::HyprvisorResult, global::BUFFER_SIZE, ipc::*};

pub async fn start_hyprland_listener() -> HyprvisorResult<()> {
    let event_socket = hyprland_socket(&HyprSocketType::Event);

    log::info!("Start Hyprland event listener");

    let mut stream = connect_to_socket(&event_socket, 1, 100).await?;

    let mut buffer = vec![0; *BUFFER_SIZE];

    loop {
        match fetch_hyprland_event(&mut stream, &mut buffer).await {
            events if events.contains(&HyprEvent::WindowChanged) => {
                handle_window_change().await;
                handle_workspace_change().await;
            }
            events
                if events.contains_at_least(&[
                    &HyprEvent::WorkspaceCreated,
                    &HyprEvent::WorkspaceDestroyed,
                ]) =>
            {
                handle_workspace_change().await;
            }
            _ => {}
        }
    }
}

async fn handle_window_change() {
    let _ = window::handle_new_event().await;
}

async fn handle_workspace_change() {
    let _ = workspaces::handle_new_event().await;
}
