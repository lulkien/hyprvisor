mod application;
mod client;
mod common_types;
mod error;
mod hyprland;
mod ipc;
mod iwd;
mod logger;
mod opts;
mod server;
mod utils;

use crate::{
    error::{HyprvisorError, HyprvisorResult},
    logger::*,
};

use opts::{Action, CommandOpts, Opts};

#[tokio::main]
async fn main() -> HyprvisorResult<()> {
    let opts = Opts::from_env();
    run(&opts).await?;

    Ok(())
}

async fn run(opts: &Opts) -> HyprvisorResult<()> {
    let level_filter = if opts.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    match &opts.action {
        Action::Daemon => init_logger(LoggerType::Server, level_filter)?,
        Action::Command(_) => init_logger(LoggerType::Command, level_filter)?,
        _ => {}
    };

    let socket_path = utils::get_socket_path();
    let server_running = check_server_alive(&socket_path).await?;

    match &opts.action {
        Action::Daemon => {
            if server_running {
                log::error!("Server is running.");
                return Err(HyprvisorError::DaemonRunning);
            }
            server::start_server(&socket_path).await?;
        }
        Action::Command(command) => {
            if !server_running {
                log::error!("Server is not running.");
                return Err(HyprvisorError::NoDaemon);
            }
            client::send_server_command(&socket_path, command, 3).await?;
        }
        Action::Listen(subscription) => {
            application::client::start_client(subscription, level_filter).await?;
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
