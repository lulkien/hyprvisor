use std::{env, time::Duration};
use tokio::net::UnixStream;

pub fn get_socket_path() -> String {
    env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{}/hyprvisor2.sock", value))
        .unwrap_or_else(|_| "/tmp/hyprvisor2.sock".to_string())
}

#[allow(unused)]
pub fn get_hyprland_event_socket() -> Option<String> {
    match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(value) => Some(format!("/tmp/hypr/{}/.socket2.sock", value)),
        Err(_) => {
            log::error!("HYPRLAND_INSTANCE_SIGNATURE not set! (is hyprland running?)");
            None
        }
    }
}

#[allow(unused)]
pub fn get_hyprland_command_socket() -> Option<String> {
    match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(value) => Some(format!("/tmp/hypr/{}/.socket.sock", value)),
        Err(_) => {
            log::error!("HYPRLAND_INSTANCE_SIGNATURE not set! (is hyprland running?)");
            None
        }
    }
}

pub async fn try_connect(
    socket_path: &str,
    attempts: usize,
    attempt_delay: u64,
) -> Option<UnixStream> {
    for attempt in 0..attempts {
        log::debug!(
            "Connect to socket {} | Attempt: {}",
            socket_path,
            attempt + 1
        );
        if let Ok(stream) = UnixStream::connect(socket_path).await {
            return Some(stream);
        }
        tokio::time::sleep(Duration::from_millis(attempt_delay)).await;
    }
    None
}
