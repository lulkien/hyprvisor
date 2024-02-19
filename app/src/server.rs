use crate::{
    common_types::{ClientInfo, Subscriber},
    opts::ServerCommand,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    io::AsyncWriteExt,
    net::{UnixListener, UnixStream},
    sync::Mutex,
};

pub struct Server {
    socket: String,
    subscribers: Arc<Mutex<Subscriber>>,
}

impl Server {
    pub fn new(socket: &str) -> Self {
        Server {
            socket: socket.to_string(),
            subscribers: Arc::new(Mutex::new(Subscriber::new())),
        }
    }

    pub async fn start(&mut self) {
        let listener = match UnixListener::bind(&self.socket) {
            Ok(listener) => {
                log::info!("Server is binded on socket {}", self.socket);
                listener
            }
            Err(e) => {
                log::error!(
                    "Failed to bind listener on socket: {}. Error: {}",
                    self.socket,
                    e
                );
                return;
            }
        };

        while let Ok((stream, _)) = listener.accept().await {
            let subscriber_ref = Arc::clone(&self.subscribers);
            tokio::spawn(handle_new_connection(stream, subscriber_ref));
        }
    }
}

async fn handle_new_connection(mut stream: UnixStream, subscribers_ref: Arc<Mutex<Subscriber>>) {
    let mut buffer: [u8; 1024] = [0; 1024];
    let bytes_received = match stream.try_read(&mut buffer) {
        Ok(message_len) => message_len,
        Err(e) => {
            log::error!("Failed to read data from stream. Error: {e}");
            return;
        }
    };

    if bytes_received < 2 {
        log::error!("Invalid message");
        return;
    }

    let client_message = String::from_utf8_lossy(&buffer[0..bytes_received]).to_string();
    log::info!("Message from client: {}", client_message);

    let command: Option<ServerCommand> = serde_json::from_str(&client_message).unwrap_or(None);
    let client: Option<ClientInfo> = serde_json::from_str(&client_message).unwrap_or(None);

    if let Some(cmd) = command {
        match cmd {
            ServerCommand::Kill => {
                let _ = stream.write_all(b"Server is shuting down...").await;
                tokio::time::sleep(Duration::from_millis(100)).await;
                std::process::exit(0);
            }
            ServerCommand::Ping => {
                let _ = stream.write_all(b"Pong").await;
            }
        }

        return;
    }

    if let Some(client_info) = client {
        let mut subscribers = subscribers_ref.lock().await;
        subscribers
            .entry(client_info.subscription_id.clone())
            .or_insert(HashMap::new());

        log::info!(
            "Client pid {} subscribe to {}",
            client_info.process_id,
            client_info.subscription_id
        );

        if stream.write_all(b"Hello").await.is_ok() {
            log::info!("Hello");
            subscribers
                .get_mut(&client_info.subscription_id)
                .unwrap()
                .insert(client_info.process_id, stream);
        }
    }
}
