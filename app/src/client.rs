use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::opts::{ServerCommand, Subscription};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

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

    pub async fn connect(&mut self) {
        let mut connection = match try_connect(&self.socket, 5, 500).await {
            Some(connection) => connection,
            None => {
                log::error!("Failed to connect to socket: {}", self.socket);
                return;
            }
        };

        let message: String = serde_json::to_string(&self.subscription).unwrap();
        connection
            .write_all(message.as_bytes())
            .await
            .expect("Failed to write message to socket");
    }
}

async fn try_connect(socket_path: &str, attempts: usize, attempt_delay: u64) -> Option<UnixStream> {
    for attempt in 0..attempts {
        log::debug!("Attempt: {}", attempt + 1);
        if let Ok(stream) = UnixStream::connect(socket_path).await {
            return Some(stream);
        }
        tokio::time::sleep(Duration::from_millis(attempt_delay)).await;
    }
    None
}

pub async fn send_server_command(socket_path: &str, command: &ServerCommand) -> bool {
    let mut stream = match try_connect(socket_path, 5, 200).await {
        Some(stream) => stream,
        None => {
            log::warn!("Cannot connect to socket: {socket_path}");
            return false;
        }
    };

    let message = serde_json::to_string(&command).unwrap();

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
