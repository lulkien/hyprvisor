use std::{env, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::common_types::HyprSocketType;

pub fn get_socket_path() -> String {
    env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{}/hyprvisor2.sock", value))
        .unwrap_or_else(|_| "/tmp/hyprvisor2.sock".to_string())
}

pub fn get_hyprland_socket(socket_type: &HyprSocketType) -> String {
    let instance_signature = match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(var) => var,
        Err(_) => panic!("Is hyprland running?"),
    };

    let socket_name = match socket_type {
        HyprSocketType::Command => ".socket.sock",
        HyprSocketType::Event => ".socket2.sock",
    };

    format!("/tmp/hypr/{instance_signature}/{socket_name}")
}

pub async fn try_connect(
    socket_path: &str,
    max_attempts: usize,
    attempt_delay: u64,
) -> Option<UnixStream> {
    for attempt in 0..max_attempts {
        log::debug!("Try connect to {} | Attempt: {}", socket_path, attempt + 1);
        if let Ok(stream) = UnixStream::connect(socket_path).await {
            return Some(stream);
        }
        tokio::time::sleep(Duration::from_millis(attempt_delay)).await;
    }
    None
}

pub async fn write_to_socket(
    socket_path: &str,
    content: &str,
    max_attempts: usize,
    attempt_delay: u64,
) -> Option<String> {
    let mut stream = match try_connect(socket_path, max_attempts, attempt_delay).await {
        Some(stream) => stream,
        None => {
            return None;
        }
    };

    if stream.write_all(content.as_bytes()).await.is_err() {
        return None;
    }

    let mut response = vec![];
    let mut buffer: [u8; 8192] = [0; 8192];

    loop {
        match stream.read(&mut buffer).await {
            Ok(size) if size > 0 => response.append(&mut buffer[0..size].to_vec()),
            Ok(_) | Err(_) => {
                break;
            }
        }
    }

    Some(String::from_utf8(response).unwrap())
}
