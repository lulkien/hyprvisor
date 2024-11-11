use crate::{
    error::{HyprvisorError, HyprvisorResult},
    ipc::{connect_to_socket, HyprvisorSocket},
    opts::CommandOpts,
    utils::HYPRVISOR_SOCKET,
};

pub async fn ping_daemon() -> HyprvisorResult<()> {
    if std::fs::metadata(HYPRVISOR_SOCKET.as_str()).is_err() {
        log::info!("Server is not running");
        return Err(HyprvisorError::NoDaemon);
    }

    let stream = connect_to_socket(&HYPRVISOR_SOCKET, 3, 100)
        .await
        .map_err(|_| HyprvisorError::NoDaemon)?;

    stream
        .write_once(&serde_json::to_string(&CommandOpts::Ping)?)
        .await?;

    let response = stream.read_once().await?;

    log::info!(
        "Response from server: {}",
        String::from_utf8(response).unwrap()
    );

    Ok(())
}
