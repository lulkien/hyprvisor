use crate::error::{HyprvisorError, HyprvisorResult};

use humantime::format_rfc3339_seconds;
use std::{process, time::SystemTime};

#[derive(Clone, PartialEq)]
pub enum LoggerType {
    Server,
    Client,
    Command,
}

pub fn init_logger(log_type: LoggerType, filter: log::LevelFilter) -> HyprvisorResult<()> {
    let process_info = if LoggerType::Client == log_type {
        format!("({}) ", process::id())
    } else {
        String::new()
    };

    let hyprvisor_logger = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}{} [{}] {} - {}",
                process_info,
                format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(filter);

    match log_type {
        LoggerType::Server => hyprvisor_logger.chain(fern::log_file("/tmp/hyprvisor-server.log")?),
        LoggerType::Client => hyprvisor_logger.chain(fern::log_file("/tmp/hyprvisor-client.log")?),
        LoggerType::Command => hyprvisor_logger.chain(std::io::stdout()),
    }
    .apply()
    .map_err(|_| HyprvisorError::LoggerError)
}
