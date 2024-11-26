use super::{
    types::{ClientInfo, SubscriptionID},
    utils::ping_daemon,
};
use crate::{
    application::utils::HYPRVISOR_SOCKET,
    bluetooth::{self, start_bluetooth_listener},
    error::{HyprvisorError, HyprvisorResult},
    global::SUBSCRIBERS,
    hyprland::{start_hyprland_listener, window, workspaces},
    ipc::{
        message::{HyprvisorMessage, MessageType},
        HyprvisorReadSock, HyprvisorWriteSock,
    },
    opts::CommandOpts,
    wifi::{self, start_wifi_listener},
};

use humantime::format_rfc3339_seconds;
use log::LevelFilter;
use std::{
    collections::HashMap,
    fs,
    time::{Duration, SystemTime},
};
use tokio::{
    net::{UnixListener, UnixStream},
    time::sleep,
};

pub async fn start_server(filter: LevelFilter) -> HyprvisorResult<()> {
    init_logger(filter)?;

    if ping_daemon().await.is_ok() {
        return Err(HyprvisorError::DaemonRunning);
    }

    if fs::metadata(HYPRVISOR_SOCKET.as_str()).is_ok() {
        fs::remove_file(HYPRVISOR_SOCKET.as_str())?;
        log::debug!("Removed: {}", HYPRVISOR_SOCKET.as_str());
    }

    log::info!("-------------------- START HYPRVISOR DAEMON --------------------");

    tokio::spawn(start_hyprland_listener());

    tokio::spawn(start_wifi_listener());

    tokio::spawn(start_bluetooth_listener());

    listen_ipc_socket().await
}

fn init_logger(filter: LevelFilter) -> HyprvisorResult<()> {
    let logger = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} [{}] {} - {}",
                format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(filter)
        .chain(fern::log_file("/tmp/hyprvisor-server.log")?);

    let logger = if LevelFilter::Debug == filter {
        logger.chain(std::io::stdout())
    } else {
        logger
    };

    logger
        .apply()
        .map_err(|e| HyprvisorError::LoggerError(fern::InitError::SetLoggerError(e)))
}

async fn listen_ipc_socket() -> HyprvisorResult<()> {
    log::info!("Try to bind on socket: {}", HYPRVISOR_SOCKET.as_str());
    let listener = UnixListener::bind(HYPRVISOR_SOCKET.as_str())?;
    log::info!("Success");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }

    Ok(())
}

async fn handle_connection(stream: UnixStream) -> HyprvisorResult<()> {
    let message = stream.try_read_message(3).await?;
    if !message.is_valid() {
        return Err(HyprvisorError::InvalidMessage);
    }

    match message.message_type {
        MessageType::Command => process_command(stream, message).await,
        MessageType::Subscription => register_client(stream, message).await,
        MessageType::Response => {
            log::error!("Ignore Response message from client.");
            Err(HyprvisorError::InvalidMessage)
        }
    }
}

async fn process_command(stream: UnixStream, message: HyprvisorMessage) -> HyprvisorResult<()> {
    if message.header != size_of::<u8>() {
        return Err(HyprvisorError::InvalidMessage);
    }

    match CommandOpts::try_from(message.payload[0])? {
        CommandOpts::Ping => {
            stream.write_message(HyprvisorMessage::from("Pong")).await?;
        }
        CommandOpts::Kill => {
            stream
                .write_message(HyprvisorMessage::from("Server is shutting down..."))
                .await?;

            sleep(Duration::from_millis(100)).await;
            std::process::exit(0);
        }
    }

    Ok(())
}

async fn register_client(stream: UnixStream, message: HyprvisorMessage) -> HyprvisorResult<()> {
    let client_info = ClientInfo::try_from(message.payload.as_slice())?;

    log::info!(
        "Client pid {} subscribe to {}",
        client_info.process_id,
        client_info.subscription_id
    );

    match client_info.subscription_id {
        SubscriptionID::Window => {
            window::response_to_subscription(&stream).await?;
        }

        SubscriptionID::Workspaces => {
            workspaces::response_to_subscription(&stream).await?;
        }

        SubscriptionID::Wifi => {
            wifi::response_to_subscription(&stream).await?;
        }

        SubscriptionID::Bluetooth => {
            bluetooth::response_to_subscription(&stream).await?;
        }

        SubscriptionID::Invalid => {
            return Err(HyprvisorError::InvalidSubscription);
        }
    };

    let (_, writer) = stream.into_split();

    //let subscribers = SUBSCRIBERS.clone();

    let mut subscribers_ref = SUBSCRIBERS.lock().await;
    subscribers_ref
        .entry(client_info.subscription_id)
        .or_insert(HashMap::new());

    subscribers_ref
        .get_mut(&client_info.subscription_id)
        .unwrap()
        .insert(client_info.process_id, writer);

    log::info!("Client connected.");

    Ok(())
}
