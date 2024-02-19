mod client;
mod common_types;
mod opts;
mod server;
mod server_response;
mod utils;

use opts::{Action, Opts, ServerCommand, SubscriptionOpts};
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
            log::info!("Start server");
            let mut server = server::Server::new(&socket_path);
            server.start().await;
        }
        Action::Command(command) => {
            if !server_running {
                return;
            }
            client::send_server_command(&socket_path, command, 3).await;
        }
        Action::Listen(subs_info) => {
            if !server_running {
                return;
            }
            let subscription = match subs_info {
                SubscriptionOpts::Workspaces { fix_workspace } => SubscriptionOpts::Workspaces {
                    fix_workspace: fix_workspace
                        .map(|fw| {
                            log::warn!(
                                "I don't think have more than 10 fixed workspaces is a good idea."
                            );
                            log::warn!("Feel free to open an issue if you have other opinion.");
                            log::warn!("https://github.com/lulkien/hyprvisor/issues");
                            Some(fw.min(10))
                        })
                        .unwrap_or(None),
                },
                SubscriptionOpts::Window { title_length } => SubscriptionOpts::Window {
                    title_length: title_length
                        .map(|tl| {
                            log::warn!(
                                "More than 100 characters for window title is too much, I think."
                            );
                            log::warn!("Feel free to open an issue if you have other opinion.");
                            log::warn!("https://github.com/lulkien/hyprvisor/issues");
                            Some(tl.min(100))
                        })
                        .unwrap_or(None),
                },
            };
            let mut client = client::Client::new(socket_path, subscription);
            client.connect().await;
        }
    };
}

async fn check_server_alive(socket_path: &str) -> bool {
    log::info!("Socket: {socket_path}");

    if fs::metadata(socket_path).is_err() {
        log::info!("Server is not running");
        return false;
    }

    let is_alive = client::send_server_command(socket_path, &ServerCommand::Ping, 1).await;

    if !is_alive {
        if let Err(e) = fs::remove_file(socket_path) {
            log::error!("Failed to remove old socket. Error: {}", e);
        } else {
            log::debug!("Removed old socket.");
        }
    }

    is_alive
}
