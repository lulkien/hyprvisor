use std::collections::HashMap;

use serde_json::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::common::{SubscriptionID, SubscriptionInfo, WindowInfo, WorkspaceInfo};

pub struct Client {
    client_info: SubscriptionInfo,
    extra_data: Option<u32>,
}

impl Client {
    pub fn new(pid: u32, subscription_id: SubscriptionID, extra_data: Option<u32>) -> Self {
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

        let subscription_msg: String = match create_subscribe_message(&self.client_info) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to create message. Error: {}", e);
                return;
            }
        };

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
                break;
            }

            let mut response_message =
                String::from_utf8_lossy(&response_buffer[..bytes_received]).to_string();

            response_message = reformat_response(
                &response_message,
                &self.client_info.subscription_id,
                &self.extra_data,
            );

            println!("{}", response_message);
        }
    }
}

fn create_subscribe_message(info: &SubscriptionInfo) -> Result<String, Error> {
    serde_json::to_string(info)
}

fn reformat_response(
    response_data: &str,
    subscription_id: &SubscriptionID,
    extra_data: &Option<u32>,
) -> String {
    if extra_data.is_none() || extra_data.unwrap() < 1 {
        return response_data.to_string();
    }

    let mut formated_response = response_data.to_string();

    match subscription_id {
        SubscriptionID::Window => {
            let mut window_info: WindowInfo = match serde_json::from_str(response_data) {
                Ok(value) => value,
                Err(_) => {
                    return "{}".to_string();
                }
            };

            let mut title = window_info.title;

            if extra_data.unwrap() > title.len() as u32 {
                return formated_response;
            }

            title = format!(
                "{}...",
                String::from_utf8_lossy(&title.as_bytes()[..extra_data.unwrap() as usize])
            );
            window_info.title = title;
            formated_response = serde_json::to_string(&window_info).unwrap();
            formated_response
        }
        SubscriptionID::Workspace => {
            let mut ws_list: Vec<WorkspaceInfo> = match serde_json::from_str(response_data) {
                Ok(list) => list,
                Err(_) => {
                    return formated_response;
                }
            };

            if extra_data.unwrap() < ws_list.len() as u32 {
                return formated_response;
            }

            let mut ws_table: HashMap<u32, WorkspaceInfo> = HashMap::new();
            for ws in &ws_list {
                ws_table.insert(ws.id, ws.clone());
            }

            for id in 1..=extra_data.unwrap() {
                if !ws_table.contains_key(&id) {
                    let empty_ws = WorkspaceInfo {
                        id,
                        name: id.to_string(),
                        occupied: false,
                        active: false,
                    };
                    ws_table.insert(id, empty_ws);
                }
            }

            ws_list.clear();
            for (_, ws) in ws_table.into_iter() {
                ws_list.push(ws);
            }
            ws_list.sort_by(|a, b| a.id.cmp(&b.id));

            formated_response = serde_json::to_string(&ws_list).unwrap();
            formated_response
        }
        _ => {
            formated_response = response_data.to_string();
            formated_response
        }
    }
}
