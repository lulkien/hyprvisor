use super::types::{HyprEventList, HyprSocketType};
use crate::{ipc::*, HyprvisorResult};

use std::env;
use tokio::{io::AsyncReadExt, net::UnixStream};

pub const HYPRLAND_SOCKET_CONNECT_ATTEMPT: u8 = 3;
pub const HYPRLAND_SOCKET_CONNECT_DELAY: u64 = 100;

pub(super) fn hyprland_socket(socket_type: &HyprSocketType) -> String {
    let instance_signature = match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(var) => var,
        Err(_) => panic!("Is hyprland running?"),
    };

    let socket_name = match socket_type {
        HyprSocketType::Command => ".socket.sock",
        HyprSocketType::Event => ".socket2.sock",
    };

    env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{value}/hypr/{instance_signature}/{socket_name}"))
        .unwrap_or_else(|_| format!("/tmp/hypr/{instance_signature}/{socket_name}"))
}

pub(super) async fn send_hyprland_command(cmd: &str) -> HyprvisorResult<Vec<u8>> {
    connect_to_socket(
        &hyprland_socket(&HyprSocketType::Command),
        HYPRLAND_SOCKET_CONNECT_ATTEMPT,
        HYPRLAND_SOCKET_CONNECT_DELAY,
    )
    .await?
    .write_and_read_multiple(cmd, 10)
    .await
}

pub(super) async fn fetch_hyprland_event(
    stream: &mut UnixStream,
    buffer: &mut [u8],
) -> HyprEventList {
    match stream.read(buffer).await {
        Ok(bytes) if bytes > 0 => buffer[..bytes].into(),
        Ok(_) | Err(_) => {
            log::error!("Connection closed from Hyprland event socket");
            std::process::exit(1);
        }
    }
}
