use std::ops::Add;
use std::{env, fs, process};

mod client;
use client::Client;

#[tokio::main]
async fn main() {
    // Get subscription id from arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <subscription_id>", args[0]);
        return;
    }

    let client_pid: u32 = process::id();
    let subscription_name = &args[1];

    // Connect to the Unix socket
    let socket_path: String = match find_socket_path() {
        Some(path) => path,
        None => {
            eprintln!("Socket not found...");
            return;
        }
    };

    let client = Client::new(client_pid, subscription_name.to_string()).await;
    client.connect(socket_path).await;
}

fn find_socket_path() -> Option<String> {
    let socket_path: String = match env::var("XDG_RUNTIME_DIR") {
        Ok(value) => value.add("/hyprvisor.sock"),
        Err(_) => "/tmp/hyprvisor.sock".to_string(),
    };

    if fs::metadata(&socket_path).is_ok() {
        Some(socket_path)
    } else {
        None
    }
}
