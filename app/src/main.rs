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

fn init_server_logger(filter: &log::LevelFilter) -> HyprvisorResult<()> {
    let log_init_result = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {} - {}",
                format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(*filter)
        .chain(std::io::stdout())
        .chain(fern::log_file("/tmp/hyprvisor-server.log")?)
        .apply();

    log_init_result.map_err(|_| HyprvisorError::LoggerError)
}

fn init_client_logger(filter: &log::LevelFilter) -> HyprvisorResult<()> {
    let log_init_result = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] ({}) {} - {}",
                format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                process::id(),
                record.target(),
                message
            ))
        })
        .level(*filter)
        .chain(fern::log_file("/tmp/hyprvisor-client.log")?)
        .apply();

    log_init_result.map_err(|_| HyprvisorError::LoggerError)
}

fn init_command_logger(filter: &log::LevelFilter) -> HyprvisorResult<()> {
    let log_init_result = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {} - {}",
                format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(*filter)
        .chain(std::io::stdout())
        .apply();

    log_init_result.map_err(|_| HyprvisorError::LoggerError)
}

async fn run(opts: &Opts) -> HyprvisorResult<()> {
    if opts.verbose {
        match opts.action {
            Action::Daemon => init_server_logger(&log::LevelFilter::Debug)?,
            Action::Listen(_) => init_client_logger(&log::LevelFilter::Debug)?,
            Action::Command(_) => init_command_logger(&log::LevelFilter::Debug)?,
        }
    } else {
        match opts.action {
            Action::Daemon => init_server_logger(&log::LevelFilter::Info)?,
            Action::Listen(_) => init_client_logger(&log::LevelFilter::Warn)?,
            Action::Command(_) => init_command_logger(&log::LevelFilter::Info)?,
        }
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
