mod client;
mod common_types;
mod error;
mod hyprland_listener;
mod opts;
mod server;
mod utils;

use crate::error::{HyprvisorError, HyprvisorResult};
use opts::{Action, CommandOpts, Opts};
use std::fs;
use utils::get_socket_path;

#[tokio::main]
async fn main() {
    let opts = Opts::from_env();
    let _ = run(&opts).await;
}

async fn run(opts: &Opts) -> HyprvisorResult<()> {
    let log_filter = if opts.debug {
        log::LevelFilter::Debug
    } else {
        match opts.action {
            Action::Listen(_) => log::LevelFilter::Error,
            _ => log::LevelFilter::Warn,
        }
    };

    pretty_env_logger::formatted_timed_builder()
        .filter(Some("hyprvisor"), log_filter)
        .init();

    let socket_path = get_socket_path();
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

    if fs::metadata(socket_path).is_err() {
        log::info!("Server is not running");
        return Ok(false);
    }

    if client::send_server_command(socket_path, &CommandOpts::Ping, 1)
        .await
        .is_err()
    {
        if let Err(e) = fs::remove_file(socket_path) {
            log::error!("Failed to remove old socket. Error: {}", e);
        } else {
            log::debug!("Removed old socket.");
            return Ok(false);
        }
    }
    Ok(true)
}
