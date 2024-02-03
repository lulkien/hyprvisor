use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

use crate::common::{HyprvisorListener, Subscribers, SubscriptionID, SubscriptionInfo};
use crate::hypr_listener::HyprListener;

pub struct Server {
    socket: String,
    subscribers: Arc<Mutex<Subscribers>>,
    is_ready: Option<bool>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            socket: "".to_string(),
            subscribers: Arc::new(Mutex::new(Subscribers::new())),
            is_ready: None,
        }
    }

    pub async fn prepare(&mut self, socket: String) {
        self.socket = socket;

        if fs::metadata(&self.socket).is_err() {
            println!("No running server binded on socket {}", self.socket);
            self.is_ready = Some(true);
            return;
        };

        match UnixStream::connect(&self.socket).await {
            Ok(_) => {
                eprintln!("There is a running server bind on {}", self.socket);
                self.is_ready = Some(false);
                return;
            }
            _ => match fs::remove_file(&self.socket) {
                Ok(_) => {
                    println!("Remove old socket {}", self.socket);
                    self.is_ready = Some(true);
                    return;
                }
                Err(e) => {
                    println!(
                        "Failed to remove old socket path {} | Error: {}",
                        self.socket, e
                    );
                    self.is_ready = Some(false);
                    return;
                }
            },
        }
    }

    pub async fn start(&mut self) {
        if self.is_ready.is_none() || Some(false) == self.is_ready {
            eprintln!("Error: Server is not ready!");
            return;
        }

        let unix_listener = match UnixListener::bind(&self.socket) {
            Ok(listener) => {
                println!("Server is listening for connection on {}", self.socket);
                listener
            }
            Err(e) => {
                eprintln!("Failed to bind on socket: {} | Error: {}", self.socket, e);
                return;
            }
        };

        let subscribers_ref = Arc::clone(&self.subscribers);
        tokio::spawn(start_listen_hypr_event(subscribers_ref));

        // Main loop
        while let Ok((stream, _)) = unix_listener.accept().await {
            let sub_ref = Arc::clone(&self.subscribers);
            tokio::spawn(handle_new_connection(stream, sub_ref));
        }
    }
}

async fn handle_new_connection(stream: UnixStream, subscribers: Arc<Mutex<Subscribers>>) {
    // Handle new connection
    let mut buffer: [u8; 1024] = [0; 1024];
    let bytes_received = match stream.try_read(&mut buffer) {
        Ok(message_len) => message_len,
        Err(e) => {
            eprintln!("Failed to read data from client. | Error: {}", e);
            return;
        }
    };

    if bytes_received < 2 {
        eprintln!("Invalid message.");
        return;
    }

    let subscription_info: Result<SubscriptionInfo, serde_json::Error> =
        serde_json::from_slice(&buffer[0..bytes_received].to_vec());

    match subscription_info {
        Ok(info) => {
            let subscription_id = match info.subscription_id {
                SubscriptionID::Workspace => SubscriptionID::Workspace,
                SubscriptionID::Window => SubscriptionID::Window,
                _ => {
                    println!("Subscription ID is not supported!");
                    return;
                }
            };

            let mut subscribers = subscribers.lock().await;
            subscribers.entry(subscription_id).or_insert(HashMap::new());

            println!(
                "New client with PID {} subscribed to {}",
                info.pid, info.subscription_id
            );

            subscribers
                .get_mut(&subscription_id)
                .unwrap()
                .insert(info.pid, stream);
        }

        Err(e) => {
            eprintln!("Failed to parse subscription message. | Error: {}", e);
            return;
        }
    }
}

async fn start_listen_hypr_event(subscribers: Arc<Mutex<Subscribers>>) {
    let mut hypr_listener: HyprListener = HyprListener::new();
    hypr_listener.prepare_listener();
    hypr_listener.start_listener(subscribers).await;
}
