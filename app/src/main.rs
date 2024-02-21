mod client;
mod common_types;
mod hyprland_listener;
mod opts;
mod server;
mod utils;

use opts::{Action, CommandOpts, Opts};
use std::fs;
use utils::get_socket_path;

#[tokio::main]
async fn main() {
    let opts = Opts::from_env();
    run(&opts).await;
}

async fn run(opts: &Opts) {
    let log_filter = if opts.debug {
        log::LevelFilter::Debug
    } else {
        match opts.action {
            Action::Listen(_) => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        }
    };

    pretty_env_logger::formatted_timed_builder()
        .filter(Some("hyprvisor"), log_filter)
        .init();

    let socket_path = get_socket_path();
    let server_running = check_server_alive(&socket_path).await;

    match &opts.action {
        Action::Daemon => {
            if server_running {
                log::error!("Server is running.");
                return;
            }
            server::start_server(&socket_path).await;
        }
        Action::Command(command) => {
            if !server_running {
                log::error!("Server is not running.");
                return;
            }
            client::send_server_command(&socket_path, command, 3).await;
        }
        Action::Listen(subscription) => {
            if !server_running {
                log::error!("Server is not running.");
                return;
            }
            client::start_client(&socket_path, subscription).await;
        }
    };
}

async fn check_server_alive(socket_path: &str) -> bool {
    log::info!("Socket: {socket_path}");

    if fs::metadata(socket_path).is_err() {
        log::info!("Server is not running");
        return false;
    }

    let is_alive = client::send_server_command(socket_path, &CommandOpts::Ping, 1).await;
    if !is_alive {
        if let Err(e) = fs::remove_file(socket_path) {
            log::error!("Failed to remove old socket. Error: {}", e);
        } else {
            log::debug!("Removed old socket.");
        }
    }

    is_alive
}
