use once_cell::sync::Lazy;
use std::env;

use crate::{client, error::HyprvisorResult, opts::CommandOpts};

pub static HYPRVISOR_SOCKET: Lazy<String> = Lazy::new(|| get_socket_path());

pub fn get_socket_path() -> String {
    match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(var) => var,
        Err(_) => panic!("Is hyprland running?"),
    };

    env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{value}/hyprvisor.sock"))
        .unwrap_or_else(|_| "/tmp/hyprvisor.sock".to_string())
}

async fn check_server_alive(socket_path: &str) -> HyprvisorResult<bool> {
    log::info!("Socket: {socket_path}");

    if std::fs::metadata(socket_path).is_err() {
        log::info!("Server is not running");
        return Ok(false);
    }

    if client::send_server_command(socket_path, &CommandOpts::Ping, 3)
        .await
        .is_err()
    {
        if let Err(e) = std::fs::remove_file(socket_path) {
            log::error!("Failed to remove old socket. Error: {}", e);
        } else {
            log::debug!("Removed old socket.");
            return Ok(false);
        }
    }
    Ok(true)
}
