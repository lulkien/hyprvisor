mod client;
mod common_types;
mod error;
mod hyprland_listener;
mod opts;
mod server;
mod utils;

use humantime::format_rfc3339_seconds;
use std::{process, time::SystemTime};

use crate::error::{HyprvisorError, HyprvisorResult};
use opts::{Action, CommandOpts, Opts};

#[tokio::main]
async fn main() -> HyprvisorResult<()> {
    let opts = Opts::from_env();
    run(&opts).await?;

    Ok(())
}

#[derive(Clone, PartialEq)]
enum LoggerType {
    Server,
    Client,
    Command,
}

fn init_logger(log_type: LoggerType, filter: log::LevelFilter) -> HyprvisorResult<()> {
    let fern_dispatch = fern::Dispatch::new();

    let process_info = if LoggerType::Client == log_type {
        format!("({}) ", process::id())
    } else {
        "".to_string()
    };

    let hyprvisor_logger = fern_dispatch
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

    let log_file_path = match log_type {
        LoggerType::Server => "/tmp/hyprvisor-server.log",
        LoggerType::Client => "/tmp/hyprvisor-client.log",
        LoggerType::Command => "",
    };

    let hyprvisor_logger = if filter == log::LevelFilter::Debug || log_type == LoggerType::Command {
        hyprvisor_logger.chain(std::io::stdout())
    } else {
        hyprvisor_logger
    };

    if log_file_path.is_empty() {
        hyprvisor_logger.apply()
    } else {
        hyprvisor_logger
            .chain(fern::log_file(log_file_path)?)
            .apply()
    }
    .map_err(|_| HyprvisorError::LoggerError)
}

async fn run(opts: &Opts) -> HyprvisorResult<()> {
    let level_filter = if opts.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    match &opts.action {
        Action::Daemon => init_logger(LoggerType::Server, level_filter)?,
        Action::Listen(_) => init_logger(LoggerType::Client, level_filter)?,
        Action::Command(_) => init_logger(LoggerType::Command, level_filter)?,
    };

    let socket_path = utils::get_socket_path();
    let server_running = check_server_alive(&socket_path).await?;

    match &opts.action {
        Action::Daemon => {
            if server_running {
                log::error!("Server is running.");
                return Err(HyprvisorError::DaemonRunning);
            }
            server::start_server(&socket_path).await;
        }
        Action::Command(command) => {
            if !server_running {
                log::error!("Server is not running.");
                return Err(HyprvisorError::NoDaemon);
            }
            client::send_server_command(&socket_path, command, 3).await?;
        }
        Action::Listen(subscription) => {
            if !server_running {
                log::error!("Server is not running.");
                return Err(HyprvisorError::NoDaemon);
            }
            client::start_client(&socket_path, subscription).await?;
        }
    };

    Ok(())
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
