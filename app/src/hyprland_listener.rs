use crate::{common_types::Subscriber, error::HyprvisorResult, utils};
use std::sync::Arc;
use tokio::{io::AsyncReadExt, sync::Mutex};

pub mod types;
pub mod window;
pub mod workspaces;
use types::{HyprEvent, HyprSocketType, HyprWinInfo, HyprWorkspaceInfo};

pub(crate) async fn start_hyprland_listener(
    subscribers: Arc<Mutex<Subscriber>>,
) -> HyprvisorResult<()> {
    let event_socket = utils::get_hyprland_socket(&HyprSocketType::Event);
    let mut current_win_info = HyprWinInfo::default();
    let mut current_ws_info: Vec<HyprWorkspaceInfo> = Vec::new();

    log::info!("Start Hyprland event listener");
    let mut event_listener = match utils::try_connect(&event_socket, 1, 500).await {
        Ok(stream) => stream,
        Err(e) => {
            log::error!("{e}");
            std::process::exit(1);
        }
    };
    let mut buffer: [u8; 8192] = [0; 8192];

    loop {
        match event_listener.read(&mut buffer).await {
            Ok(bytes) if bytes > 0 => {
                let events = parse_events(&buffer[..bytes]);
                log::info!("{:?}", events);
                if events.contains(&HyprEvent::WindowChanged) {
                    send_window_info(&mut current_win_info, subscribers.clone()).await;
                    send_workspace_info(&mut current_ws_info, subscribers.clone()).await;
                } else if events.contains(&HyprEvent::WorkspaceCreated)
                    || events.contains(&HyprEvent::WorkspaceDestroyed)
                {
                    send_workspace_info(&mut current_ws_info, subscribers.clone()).await;
                }
            }

            Ok(_) | Err(_) => {
                log::error!("Connection closed from Hyprland event socket");
                std::process::exit(1);
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

async fn send_window_info(current_info: &mut HyprWinInfo, subscribers: Arc<Mutex<Subscriber>>) {
    if let Err(e) = window::broadcast_info(current_info, subscribers.clone()).await {
        log::info!("Window: {e}");
    }
}

async fn send_workspace_info(
    current_info: &mut Vec<HyprWorkspaceInfo>,
    subscribers: Arc<Mutex<Subscriber>>,
) {
    if let Err(e) = workspaces::broadcast_info(current_info, subscribers.clone()).await {
        log::info!("Workspace: {e}");
    }
}
