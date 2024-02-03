use serde_json::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::common::{SubscriptionID, SubscriptionInfo};

pub struct Client {
    client_info: SubscriptionInfo,
    extra_data: Option<u8>,
}

#[allow(unreachable_code, unused_variables)]
#[allow(unused_mut)]
impl Client {
    pub fn new(pid: u32, subscription_id: SubscriptionID, extra_data: Option<u8>) -> Self {
        Client {
            client_info: SubscriptionInfo {
                pid,
                subscription_id,
            },
            extra_data,
        }
    }

    pub async fn connect(self, socket_path: String) {
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
        println!("Send: {}", subscription_msg);

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
                    println!("Error reading from server: {}", e);
                    break;
                }
            };

            if bytes_received == 0 {
                println!("Server closed the connection");
                break;
            }

            let response_message = String::from_utf8_lossy(&response_buffer[..bytes_received]);
            println!("{}", response_message);
        }
    }
}

fn create_subscribe_message(info: SubscriptionInfo) -> Result<String, Error> {
    serde_json::to_string(&info)
}
