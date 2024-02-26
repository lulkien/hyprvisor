use crate::{
    common_types::{ClientInfo, Subscriber, SubscriptionID},
    hyprland_listener,
    opts::CommandOpts,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    io::AsyncWriteExt,
    net::{UnixListener, UnixStream},
    sync::Mutex,
};

pub async fn start_server(socket: &str) {
    log::info!("Starting hyprvisor server...");
    let subscribers = Arc::new(Mutex::new(Subscriber::new()));

    // Start hyprland listener thread
    tokio::spawn(hyprland_listener::start_hyprland_listener(
        subscribers.clone(),
    ));

    let listener = match UnixListener::bind(socket) {
        Ok(listener) => {
            log::info!("Server is binded on socket {}", socket);
            listener
        }
        Err(e) => {
            log::error!(
                "Failed to bind listener on socket: {}. Error: {}",
                socket,
                e
            );
            return;
        }
    };

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, subscribers.clone()));
    }
}

async fn handle_connection(mut stream: UnixStream, subscribers_ref: Arc<Mutex<Subscriber>>) {
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

    let command: Option<CommandOpts> = serde_json::from_str(&client_message).unwrap_or(None);
    let client: Option<ClientInfo> = serde_json::from_str(&client_message).unwrap_or(None);

    if let Some(cmd) = command {
        match cmd {
            CommandOpts::Kill => {
                let shutdown_message = "Server is shuting down...".to_string();
                log::warn!("{shutdown_message}");
                stream.write_all(shutdown_message.as_bytes()).await.unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                std::process::exit(0);
            }
            CommandOpts::Ping => {
                stream.write_all(b"Pong").await.unwrap();
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

        let message = match client_info.subscription_id {
            SubscriptionID::Window => {
                match hyprland_listener::window::get_hypr_active_window().await {
                    Ok(win_info) => serde_json::to_string(&win_info),
                    Err(_) => return,
                }
            }
            SubscriptionID::Workspaces => {
                match hyprland_listener::workspaces::get_hypr_workspace_info().await {
                    Ok(ws_info) => serde_json::to_string(&ws_info),
                    Err(_) => return,
                }
            }
        };

        match message {
            Ok(msg) => {
                if stream.write_all(msg.as_bytes()).await.is_ok() {
                    log::info!("Client connected.");
                    subscribers
                        .get_mut(&client_info.subscription_id)
                        .unwrap()
                        .insert(client_info.process_id, stream);
                }
            }
            Err(e) => {
                log::error!("Failed to serialize message. Error: {}", e);
            }
        }
    }
}
