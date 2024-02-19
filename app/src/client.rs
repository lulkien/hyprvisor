use crate::{
    common_types::{ClientInfo, SubscriptionID},
    opts::{ServerCommand, SubscriptionOpts},
};
use std::{process, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

#[allow(unused)]
pub struct Client {
    socket: String,
    client_info: ClientInfo,
    extra_data: Option<u32>,
}

impl Client {
    pub fn new(socket: String, subscription: SubscriptionOpts) -> Self {
        let (subscription_id, subscription_ext) = match subscription {
            SubscriptionOpts::Workspaces { fix_workspace } => {
                (SubscriptionID::Workspaces, fix_workspace)
            }
            SubscriptionOpts::Window { title_length } => (SubscriptionID::Window, title_length),
        };
        let process_id = process::id();
        Client {
            socket,
            client_info: ClientInfo::new(process_id, subscription_id),
            extra_data: subscription_ext,
        }
    }

    pub async fn connect(&mut self) {
        let max_tries = 5;
        let delay = 500;
        let mut connection = match try_connect(&self.socket, max_tries, delay).await {
            Some(connection) => connection,
            None => {
                log::error!("Failed to connect to socket: {}", self.socket);
                return;
            }
        };

        let message: String = serde_json::to_string(&self.client_info).unwrap();
        connection
            .write_all(message.as_bytes())
            .await
            .expect("Failed to write message to socket");

        loop {
            let mut response_buffer: [u8; 8192] = [0; 8192];
            let bytes_received = match connection.read(&mut response_buffer).await {
                Ok(size) if size > 0 => size,
                Ok(_) | Err(_) => {
                    log::error!("Error reading from server.");
                    return;
                }
            };

            let response_message =
                String::from_utf8_lossy(&response_buffer[..bytes_received]).to_string();

            // response_message = reformat_response(
            //     &response_message,
            //     &self.client_info.subscription_id,
            //     &self.extra_data,
            // );

            println!("{response_message}");
        }
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

pub async fn send_server_command(
    socket_path: &str,
    command: &ServerCommand,
    max_tries: usize,
) -> bool {
    let delay = 200;
    let mut stream = match try_connect(socket_path, max_tries, delay).await {
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
