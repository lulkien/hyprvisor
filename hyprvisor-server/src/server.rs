use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::common::{Subscribers, SubscriptionID, SubscriptionInfo};

pub struct Server {
    socket_path: String,
    listener: Option<UnixListener>,
    subscribers: Arc<Mutex<Subscribers>>,
}

impl Server {
    pub async fn new(socket_path: String) -> Self {
        Server {
            socket_path,
            listener: None,
            subscribers: Arc::new(Mutex::new(Subscribers::new())),
        }
    }

    pub async fn prepare(&mut self) {
        if fs::metadata(&self.socket_path).is_err() {
            println!("No running server binded on socket {}", self.socket_path);
            self.create_new_listener();
            return;
        };

        match UnixStream::connect(&self.socket_path).await {
            Ok(_) => {
                eprintln!("There is a running server binded on {}", self.socket_path);
                self.listener = None;
                return;
            }
            _ => match fs::remove_file(&self.socket_path) {
                Ok(_) => {
                    println!("Removed old socket {}", self.socket_path);
                    self.create_new_listener();
                    return;
                }
                Err(e) => {
                    println!(
                        "Failed to remove old socket {}. Error: {}",
                        self.socket_path, e
                    );
                    self.listener = None;
                    return;
                }
            },
        }
    }

    fn create_new_listener(&mut self) {
        self.listener = match UnixListener::bind(&self.socket_path) {
            Ok(listener) => {
                println!("Listening for connection on {}", self.socket_path);
                Some(listener)
            }
            Err(err) => {
                eprintln!(
                    "Failed to bind on socket {}. Error: {}",
                    self.socket_path, err
                );
                None
            }
        };
    }

    pub async fn start(&mut self) {
        if self.listener.is_none() {
            return;
        }

        let subscribers_ref = Arc::clone(&self.subscribers);
        tokio::spawn(broadcast_data(subscribers_ref));

        // Main loop
        let listener = self.listener.as_ref().unwrap();
        while let Ok((stream, _)) = listener.accept().await {
            let sub_ref = Arc::clone(&self.subscribers);
            tokio::spawn(handle_new_connection(stream, sub_ref));
        }
    }
}

async fn handle_new_connection(mut stream: UnixStream, subscribers: Arc<Mutex<Subscribers>>) {
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
            let subscription_id = match info.name.as_str() {
                "workspace" => SubscriptionID::WORKSPACE,
                "window" => SubscriptionID::WINDOW,
                "sink_volume" => SubscriptionID::SINKVOLUME,
                "source_volume" => SubscriptionID::SOURCEVOLUME,
                _ => {
                    eprintln!("Invalid subscription");
                    return;
                }
            };

            let mut subscribers = subscribers.lock().await;
            subscribers.entry(subscription_id).or_insert(HashMap::new());

            println!(
                "New client with PID {} subscribed to {}",
                info.pid, info.name
            );

            let response_message = "From server with love".to_string();
            stream.write_all(response_message.as_bytes()).await.unwrap();

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

async fn broadcast_data(subscribers: Arc<Mutex<Subscribers>>) {
    loop {
        println!("Send message to client");
        {
            // Lock server state
            let mut subscribers = subscribers.lock().await;

            for (_, subscribers) in subscribers.iter_mut() {
                let mut disconnected_pid: Vec<u32> = Vec::new();
                for (pid, stream) in subscribers.iter_mut() {
                    let msg = "Test connection".to_string();
                    match stream.write_all(msg.as_bytes()).await {
                        Ok(_) => {
                            println!("Client {} is alive.", pid);
                        }
                        Err(e) => {
                            println!("Client {} is no longer alive. Error: {}", pid, e);
                            disconnected_pid.push(pid.clone());
                        }
                    }
                }

                // Remove disconnected_pid
                for pid in disconnected_pid {
                    subscribers.remove(&pid);
                }
            }
            // Release server state
        }

        sleep(Duration::from_secs(2)).await;
    }
}
