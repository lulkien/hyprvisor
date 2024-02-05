use clap::Parser;
use std::{env, process};

mod common;
use common::{Cli, Commands, SubscriptionID};

mod client;
use client::Client;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Prepare client data
    let client_pid: u32 = process::id();
    let mut extra_data: Option<u32> = None;
    let sub_id = match &cli.command {
        Commands::Workspace { fix } => {
            extra_data = *fix;
            SubscriptionID::Workspace
        }
        Commands::Window { max_char } => {
            extra_data = *max_char;
            SubscriptionID::Window
        }
        Commands::SinkVolume => SubscriptionID::SinkVolume,
        Commands::SourceVolume => SubscriptionID::SourceVolume,
    };

    // Connect to the Unix socket
    let socket_path: String = env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{}/hyprvisor.sock", value))
        .unwrap_or_else(|_| "/tmp/hyprvisor.sock".to_string());

    let client = Client::new(client_pid, sub_id, extra_data);
    client.connect(socket_path).await;
}
