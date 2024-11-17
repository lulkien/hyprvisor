use std::time::SystemTime;

use humantime::format_rfc3339_seconds;
use log::LevelFilter;

use crate::{
    error::{HyprvisorError, HyprvisorResult},
    ipc::{connect_to_socket, HyprvisorRequestResponse},
    opts::CommandOpts,
};

use super::utils::HYPRVISOR_SOCKET;

pub async fn send_command(command: CommandOpts, filter: LevelFilter) -> HyprvisorResult<()> {
    init_logger(filter)?;

    let stream = connect_to_socket(&HYPRVISOR_SOCKET, 3, 100).await?;

    log::info!("Send command to server: {}", command);

    let response_message = stream.send_and_receive_message(command.into()).await?;

    if !response_message.is_valid() {
        return Err(HyprvisorError::InvalidResponse);
    }

    log::info!(
        "Response from server: {}",
        String::from_utf8(response_message.payload).map_err(|_| HyprvisorError::ParseError)?
    );

    Ok(())
}

fn init_logger(filter: LevelFilter) -> HyprvisorResult<()> {
    let logger = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] {} - {}",
                format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(filter)
        .chain(std::io::stdout());

    logger.apply().map_err(|_| HyprvisorError::LoggerError)
}
