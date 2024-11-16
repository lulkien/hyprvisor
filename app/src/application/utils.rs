use crate::{
    error::{HyprvisorError, HyprvisorResult},
    ipc::{connect_to_socket, message::MessageType, HyprvisorReadSock, HyprvisorWriteSock},
    opts::CommandOpts,
};

use once_cell::sync::Lazy;
use std::env;

pub(super) static HYPRVISOR_SOCKET: Lazy<String> = Lazy::new(|| {
    match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(var) => var,
        Err(_) => panic!("Is hyprland running?"),
    };

    env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{value}/hyprvisor.sock"))
        .unwrap_or_else(|_| "/tmp/hyprvisor.sock".to_string())
});

pub(super) async fn ping_daemon() -> HyprvisorResult<()> {
    if std::fs::metadata(HYPRVISOR_SOCKET.as_str()).is_err() {
        log::info!("Server is not running");
        return Err(HyprvisorError::NoDaemon);
    }

    let stream = connect_to_socket(&HYPRVISOR_SOCKET, 3, 100)
        .await
        .map_err(|_| HyprvisorError::NoDaemon)?;

    stream.write_message(CommandOpts::Ping.into()).await?;

    let response = stream.read_message().await?;

    if response.message_type != MessageType::Response {
        return Err(HyprvisorError::InvalidResponse);
    }

    log::info!(
        "Response from server: {}",
        String::from_utf8(response.payload).map_err(|_| HyprvisorError::ParseError)?
    );

    Ok(())
}
