use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, ops::Add, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::Mutex,
};

use crate::common::{HyprvisorListener, Subscribers, SubscriptionID};

#[derive(Debug, Deserialize, Serialize)]
struct WorkspaceInfo {
    id: String,
    name: String,
    monitor: String,
    active: bool,
}

impl WorkspaceInfo {
    fn new() -> Self {
        WorkspaceInfo {
            id: "".to_string(),
            name: "".to_string(),
            monitor: "".to_string(),
            active: false,
        }
    }
}

pub struct HyprListener {
    listen_sock: Option<String>,
    command_sock: Option<String>,
}

impl HyprListener {
    pub fn new() -> Self {
        HyprListener {
            listen_sock: None,
            command_sock: None,
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

    async fn start_listener(&mut self, subscribers: Arc<Mutex<Subscribers>>) {
        if self.listen_sock.is_none() || self.command_sock.is_none() {
            eprintln!("Failed to get hyprland socket");
            return;
        }

        let listen_socket = self.listen_sock.as_ref().unwrap();
        let command_socket = self.command_sock.as_ref().unwrap();

        let mut stream = match UnixStream::connect(listen_socket).await {
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
                    let data_changed = process_event(&buffer[..size]);
                    if data_changed.0 {
                        let subscribers_ref = Arc::clone(&subscribers);
                        broadcast_workspace_info(command_socket.to_string(), subscribers_ref).await;
                    }
                }
                Ok(_) | Err(_) => {
                    eprintln!("Connection close from socket");
                    break;
                }
            }
        }
    }
}

fn process_event(buffer: &[u8]) -> (bool, bool) {
    let data_chunk = String::from_utf8_lossy(buffer);
    let events: Vec<String> = data_chunk.lines().map(String::from).collect();
    let mut data_changed: (bool, bool) = (false, false);

    for event in events {
        match event {
            e if e.contains("workspace>>") => data_changed.0 = true,
            e if e.contains("activewindow>>") => data_changed.1 = true,
            _ => { /* ignore */ }
        }
    }

    data_changed
}

async fn broadcast_workspace_info(cmd_sock: String, subscribers: Arc<Mutex<Subscribers>>) {
    let mut command_stream: UnixStream = match UnixStream::connect(&cmd_sock).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!(
                "Failed to connect to command socket: {}. Error: {}",
                cmd_sock, e
            );
            return;
        }
    };

    command_stream
        .write_all(b"workspaces")
        .await
        .expect("Failed to write subscription type");

    let mut buffer: [u8; 10240] = [0; 10240];
    let mut line = String::new();
    let mut ws_info: Vec<WorkspaceInfo> = Vec::new();

    loop {
        match command_stream.read(&mut buffer).await {
            Ok(size) if size > 0 => {
                let chunk_str = String::from_utf8_lossy(&buffer[..size]);
                for ch in chunk_str.chars() {
                    match ch {
                        '\n' => {
                            match extract_ws_info(&line.trim().to_string()) {
                                Some(ws) => {
                                    ws_info.push(ws);
                                }
                                None => {}
                            }
                            line.clear();
                        }
                        _ => line.push(ch),
                    }
                }
            }
            Ok(_) | Err(_) => break, // Break on end of response or error
        }
    }

    let ws_msg = match serde_json::to_string(&ws_info) {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    let mut disconnected_pid: Vec<u32> = Vec::new();
    let mut subscribers = subscribers.lock().await;
    let ws_subscriber: &mut HashMap<u32, UnixStream> =
        subscribers.get_mut(&SubscriptionID::WORKSPACE).unwrap();

    for (pid, stream) in ws_subscriber.into_iter() {
        match stream.write_all(ws_msg.as_bytes()).await {
            Ok(_) => {
                println!("Client {pid} is alive.");
            }
            Err(_) => {
                println!("Client {pid} is dead. Remove later");
                disconnected_pid.push(pid.clone());
            }
        }
    }

    for pid in disconnected_pid {
        ws_subscriber.remove(&pid);
    }

    // println!("{ws_msg}");
}

fn extract_ws_info(data: &String) -> Option<WorkspaceInfo> {
    let rgx = match Regex::new(r"workspace ID (\d+) \(([^)]+)\) on monitor ([^:]+):") {
        Ok(rgx) => rgx,
        Err(e) => {
            eprintln!("Failed to create regex pattern. Error: {}", e);
            return None;
        }
    };

    match rgx.captures(data.as_str()) {
        Some(data) => {
            let mut ws: WorkspaceInfo = WorkspaceInfo::new();
            ws.id = data[1].to_string();
            ws.name = data[2].to_string();
            ws.monitor = data[3].to_string();
            ws.active = false;
            Some(ws)
        }
        None => None,
    }
}
