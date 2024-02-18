use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::opts::{ServerCommand, Subscription};
use std::collections::{HashMap, HashSet};

pub type Subscriber = HashMap<Subscription, HashSet<UnixStream>>;

pub struct Client {
    socket: String,
    subscription: Subscription,
}

impl Client {
    pub fn new(socket: String, subscription: Subscription) -> Self {
        Client {
            socket,
            subscription,
        }
    }

    pub async fn connect(&mut self) {}
}

async fn attempt_connect(socket_path: &str, attempt: usize) -> Option<UnixStream> {
    None
}

pub async fn send_server_command(socket_path: &str, command: &ServerCommand) -> bool {
    let mut stream = match UnixStream::connect(socket_path).await {
        Ok(stream) => stream,
        Err(e) => {
            log::error!("Cannot connect to socket {socket_path}. Error: {e}");
            return false;
        }
    };

    let message = match serde_json::to_string(&command) {
        Ok(msg) => msg,
        Err(e) => {
            log::error!(
                "Failed to create message from command: {:?}. Error: {e}",
                command
            );
            return false;
        }
    };

    if let Err(e) = stream.write_all(message.as_bytes()).await {
        log::error!("Failed to write message to socket. Error: {e}");
        return false;
    }

    let mut buffer = [0; 1024];
    let bytes_received = match stream.read(&mut buffer).await {
        Ok(size) if size > 0 => size,
        Ok(_) | Err(_) => {
            log::error!("Failed to read response from server");
            return false;
        }
    };

    let response = String::from_utf8_lossy(&buffer[0..bytes_received]).to_string();
    log::info!("Response: {response}");
    true
}
