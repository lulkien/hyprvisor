use serde::{Deserialize, Serialize};
use std::{env, ops::Add, sync::Arc};
use tokio::{io::AsyncReadExt, net::UnixStream, sync::Mutex};

use crate::common::{HyprvisorListener, Subscribers};

#[derive(Debug, Deserialize, Serialize)]
struct WorkspaceInfo {
    id: String,
    name: String,
    monitor: String,
    active: bool,
}

#[allow(dead_code)]
pub struct HyprListener {
    listen_sock: Option<String>,
    command_sock: Option<String>,
    listener: Option<UnixStream>,
}

impl HyprListener {
    pub fn new() -> Self {
        HyprListener {
            listen_sock: None,
            command_sock: None,
            listener: None,
        }
    }
}

impl HyprvisorListener for HyprListener {
    fn prepare_listener(&mut self) {
        let hyprland_instance_signature = match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(value) => value,
            Err(_) => {
                eprintln!("HYPRLAND_INSTANCE_SIGNATURE not set! (is hyprland running?)");
                return;
            }
        };
        self.listen_sock = Some(
            "/tmp/hypr/"
                .to_string()
                .add(hyprland_instance_signature.as_str())
                .add("/.socket2.sock"),
        );
        self.command_sock = Some(
            "/tmp/hypr/"
                .to_string()
                .add(hyprland_instance_signature.as_str())
                .add("/.socket.sock"),
        );
    }

    async fn start_listener(&mut self, _subscribers: Arc<Mutex<Subscribers>>) {
        if self.listen_sock.is_none() || self.command_sock.is_none() {
            eprintln!("Failed to get hyprland socket");
            return;
        }

        let listen_socket = self.listen_sock.as_ref().unwrap().clone();
        // let command_socket = self.command_sock.as_ref().unwrap().clone();

        let mut stream = match UnixStream::connect(&listen_socket).await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Failed to connect to {}. Error: {}", listen_socket, e);
                return;
            }
        };

        let mut buffer: [u8; 10240] = [0; 10240];
        loop {
            match stream.read(&mut buffer).await {
                Ok(size) if size > 0 => {
                    let data_chunk: String =
                        String::from_utf8_lossy(&buffer[..size]).trim().to_string();
                    println!("-------------------\n{}\n---------------", data_chunk);
                }
                Ok(_) | Err(_) => {
                    eprintln!("Connection close from socket");
                    break;
                }
            }
        }
    }
}

#[allow(dead_code)]
async fn broadcast_workspace_info(_cmd_sock: String, _server_state: Arc<Mutex<Subscribers>>) {
    unimplemented!("TODO")
}
