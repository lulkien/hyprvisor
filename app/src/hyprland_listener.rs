use crate::{client, common_types::Subscriber, error::HyprvisorResult, opts::CommandOpts, utils};
use std::sync::Arc;
use tokio::{io::AsyncReadExt, sync::Mutex};

pub mod types;
use types::{HyprEvent, HyprSocketType, HyprWinInfo, HyprWorkspaceInfo};
mod window;
mod workspaces;

pub async fn start_hyprland_listener(subscribers: Arc<Mutex<Subscriber>>) -> HyprvisorResult<()> {
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
                    window::broadcast_info(&mut current_win_info, subscribers.clone()).await?;
                    workspaces::broadcast_info(&mut current_ws_info, subscribers.clone()).await?;
                } else if events.contains(&HyprEvent::WorkspaceCreated)
                    || events.contains(&HyprEvent::WorkspaceDestroyed)
                {
                    workspaces::broadcast_info(&mut current_ws_info, subscribers.clone()).await?;
                }
            }

            Ok(_) | Err(_) => {
                log::error!("Connection closed from Hyprland event socket");
                client::send_server_command(&utils::get_socket_path(), &CommandOpts::Kill, 1)
                    .await?;
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
