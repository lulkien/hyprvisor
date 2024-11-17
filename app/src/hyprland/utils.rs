use super::types::{HyprEventList, HyprSocketType};
use crate::{global::BUFFER_SIZE, ipc::*, HyprvisorResult};

use std::env;
use tokio::{io::AsyncReadExt, net::UnixStream};

pub const HYPRLAND_SOCKET_CONNECT_ATTEMPT: u8 = 3;
pub const HYPRLAND_SOCKET_CONNECT_DELAY: u64 = 100;

pub(super) fn hyprland_socket(socket_type: &HyprSocketType) -> String {
    let instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").expect("Is Hyprland running?");

    let socket_name = match socket_type {
        HyprSocketType::Command => ".socket.sock",
        HyprSocketType::Event => ".socket2.sock",
    };

    env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{value}/hypr/{instance_signature}/{socket_name}"))
        .unwrap_or_else(|_| format!("/tmp/hypr/{instance_signature}/{socket_name}"))
}

pub(super) async fn send_hyprland_command(command: &str) -> HyprvisorResult<Vec<u8>> {
    log::debug!("send_hyprland_command: {}", command);

    let mut buffer = vec![0; *BUFFER_SIZE];

    connect_to_socket(
        &hyprland_socket(&HyprSocketType::Command),
        HYPRLAND_SOCKET_CONNECT_ATTEMPT,
        HYPRLAND_SOCKET_CONNECT_DELAY,
    )
    .await?
    .try_send_and_receive_bytes(command.as_bytes(), &mut buffer, 10)
    .await
    .map(|recv_len| buffer[..recv_len].to_vec())
}

pub(super) async fn fetch_hyprland_event(
    stream: &mut UnixStream,
    buffer: &mut [u8],
) -> HyprEventList {
    log::debug!("fetch_hyprland_event");

    stream
        .read(buffer)
        .await
        .map(|len| buffer[..len].into())
        .expect("Connection closed from Hyprland event socket.")
}
