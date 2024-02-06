use std::collections::HashMap;
use std::time::Duration;
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

    pub async fn connect(self, socket_path: String, max_tries: usize) {
        let mut retries: usize = 0;
        while retries < max_tries {
            let mut stream: UnixStream = match UnixStream::connect(&socket_path).await {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("Failed to connect to Unix socket: {socket_path} | Error: {e}");
                    retries += 1;
                    if retries < max_tries {
                        eprintln!("Attempt {retries}/{max_tries}. Retrying...");
                        tokio::time::sleep(Duration::from_millis(300)).await;
                        continue;
                    } else {
                        eprintln!("Cannot establish connection to socket {socket_path}.");
                        return;
                    }
                }
            };

            let subscription_msg = match serde_json::to_string(&self.client_info) {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Failed to create message. Error: {}", e);
                    return;
                }
            };

            if let Err(e) = stream.write_all(subscription_msg.as_bytes()).await {
                eprintln!("Failed to write subscription type. Error: {}", e);
                return;
            }

            loop {
                let mut response_buffer: [u8; 8192] = [0; 8192];
                let bytes_received = match stream.read(&mut response_buffer).await {
                    Ok(size) if size > 0 => size,
                    Ok(_) | Err(_) => {
                        eprintln!("Error reading from server.");
                        return;
                    }
                };

                let mut response_message =
                    String::from_utf8_lossy(&response_buffer[..bytes_received]).to_string();

                response_message = reformat_response(
                    &response_message,
                    &self.client_info.subscription_id,
                    &self.extra_data,
                );

                println!("{response_message}");
            }
        }
    }
}

fn reformat_response(
    response_data: &str,
    subscription_id: &SubscriptionID,
    extra_data: &Option<u32>,
) -> String {
    if extra_data.map_or(true, |val| val < 1) {
        return response_data.to_string();
    }

    let mut formatted_response = response_data.to_string();

    match subscription_id {
        SubscriptionID::Window => {
            let window_info: Result<WindowInfo, _> = serde_json::from_str(response_data);
            let mut window_info = match window_info {
                Ok(value) => value,
                Err(_) => return "{}".to_string(),
            };
            if let Some(title) = window_info.title.get(..extra_data.unwrap() as usize) {
                window_info.title = format!("{}...", String::from_utf8_lossy(title.as_bytes()));
                formatted_response = serde_json::to_string(&window_info).unwrap();
            }
        }
        SubscriptionID::Workspace => {
            let ws_list: Result<Vec<WorkspaceInfo>, _> = serde_json::from_str(response_data);
            let mut ws_list = match ws_list {
                Ok(list) => list,
                Err(_) => return formatted_response,
            };

            if ws_list.len() > extra_data.unwrap() as usize {
                return formatted_response;
            }

            let mut ws_table: HashMap<u32, WorkspaceInfo> =
                ws_list.clone().into_iter().map(|ws| (ws.id, ws)).collect();

            for id in 1..=extra_data.unwrap() {
                ws_table.entry(id).or_insert_with(|| WorkspaceInfo {
                    id,
                    name: id.to_string(),
                    occupied: false,
                    active: false,
                });
            }

            ws_list.clear();
            ws_list = ws_table.into_iter().map(|(_, ws)| ws).collect();
            ws_list.sort_by(|a, b| a.id.cmp(&b.id));

            formatted_response = serde_json::to_string(&ws_list).unwrap();
        }
        _ => (),
    }
    formatted_response
}
