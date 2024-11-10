use crate::{
    common_types::{ClientInfo, Subscriber, SubscriptionID},
    error::HyprvisorResult,
    hyprland::{start_hyprland_listener, window, workspaces},
    ipc::*,
    opts::CommandOpts,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    net::{UnixListener, UnixStream},
    sync::Mutex,
    time::sleep,
};

pub async fn start_server(socket: &str) -> HyprvisorResult<()> {
    log::info!("--------------------------------- START HYPRVISOR DAEMON ---------------------------------");
    let subscribers = Arc::new(Mutex::new(Subscriber::new()));

    // Start hyprland listener thread
    tokio::spawn(start_hyprland_listener(subscribers.clone()));

    log::debug!("Try to bind on socket: {socket}");
    let listener = UnixListener::bind(socket)?;
    log::debug!("Success");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, subscribers.clone()));
    }
    Ok(())
}

async fn handle_connection(stream: UnixStream, subscribers_ref: Arc<Mutex<Subscriber>>) {
    let client_message: String = match stream.read_multiple(3).await {
        Ok(msg) => String::from_utf8(msg).unwrap(),
        Err(_) => return,
    };

    log::debug!("Message from client: {}", client_message);

    let command: Option<CommandOpts> = serde_json::from_str(&client_message).unwrap_or(None);

    if let Some(cmd) = command {
        process_server_command(stream, cmd).await;
        return;
    }

    let client: Option<ClientInfo> = serde_json::from_str(&client_message).unwrap_or(None);

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
            SubscriptionID::Window => match window::get_hypr_active_window().await {
                Ok(win_info) => serde_json::to_string(&win_info),
                Err(_) => return,
            },
            SubscriptionID::Workspaces => match workspaces::get_hypr_workspace_info().await {
                Ok(ws_info) => serde_json::to_string(&ws_info),
                Err(_) => return,
            },
            SubscriptionID::Wireless => {
                todo!()
            }
        };

        match message {
            Ok(msg) => {
                if stream.write_once(&msg).await.is_ok() {
                    subscribers
                        .get_mut(&client_info.subscription_id)
                        .unwrap()
                        .insert(client_info.process_id, stream);
                    log::info!("Client connected.");
                }
            }
            Err(e) => {
                log::error!("Failed to serialize message. Error: {}", e);
            }
        }
    }
}

async fn process_server_command(stream: UnixStream, cmd: CommandOpts) {
    match cmd {
        CommandOpts::Kill => {
            let shutdown_message = "Server is shuting down...";
            log::info!("{shutdown_message}");
            stream.write_once(shutdown_message).await.unwrap();
            sleep(Duration::from_millis(100)).await;
            std::process::exit(0);
        }
        CommandOpts::Ping => {
            stream.write_once("Pong").await.unwrap();
        }
    }
}
