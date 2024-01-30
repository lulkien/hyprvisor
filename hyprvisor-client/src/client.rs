use serde::{Deserialize, Serialize};
use serde_json::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Serialize, Deserialize)]
struct SubscriptionInfo {
    pid: u32,
    name: String,
}

pub struct Client {
    client_info: SubscriptionInfo,
    is_valid: bool,
}

impl Client {
    pub async fn new(pid: u32, subscription: String) -> Self {
        Client {
            client_info: SubscriptionInfo {
                pid,
                name: subscription.clone(),
            },
            is_valid: check_subscription(&subscription),
        }
    }

    pub async fn connect(self, socket_path: String) {
        if !self.is_valid {
            eprintln!("Client info is not valid.");
            return;
        }

        let mut stream = match UnixStream::connect(&socket_path).await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!(
                    "Failed to connect to Unix socket: {} | Error: {}",
                    socket_path, e
                );
                return;
            }
        };

        let subscription_msg: String = match create_subscribe_message(self.client_info) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to create message. Error: {}", e);
                return;
            }
        };
        // println!("Send: {}", subscription_msg);

        stream
            .write_all(subscription_msg.as_bytes())
            .await
            .expect("Failed to write subscription type");

        // Continuously listen for responses from the server
        loop {
            let mut response_buffer: [u8; 8192] = [0; 8192]; // Adjust the buffer size based on your expected message size
            let bytes_received = match stream.read(&mut response_buffer).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("Error reading from server: {}", e);
                    break;
                }
            };

            if bytes_received == 0 {
                eprintln!("Server closed the connection");
                break;
            }

            let response_message = String::from_utf8_lossy(&response_buffer[..bytes_received]);
            println!("{}", response_message);
        }
    }
}

fn check_subscription(id: &String) -> bool {
    let allow_id: Vec<String> = vec![
        "workspace".to_string(),
        "window".to_string(),
        "sink_volume".to_string(),
        "source_volume".to_string(),
    ];

    allow_id.contains(id)
}

fn create_subscribe_message(info: SubscriptionInfo) -> Result<String, Error> {
    serde_json::to_string(&info)
}
