use crate::{
    common_types::SubscriptionID,
    opts::{ServerCommand, Subscription},
};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

pub struct Client {
    socket: String,
    subscription_id: SubscriptionID,
    subscription_ext: Option<u32>,
}

impl Client {
    pub fn new(socket: String, subscription: Subscription) -> Self {
        let (subscription_id, subscription_ext) = match subscription {
            Subscription::Workspaces { fix_workspace } => {
                (SubscriptionID::Workspaces, fix_workspace)
            }
            Subscription::Window { title_length } => (SubscriptionID::Window, title_length),
        };
        Client {
            socket,
            subscription_id,
            subscription_ext,
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

        let message: String = serde_json::to_string(&self.subscription_id).unwrap();
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
