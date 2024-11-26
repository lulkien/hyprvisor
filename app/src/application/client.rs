use super::{
    types::{ClientInfo, SubscriptionID},
    utils::{ping_daemon, HYPRVISOR_SOCKET},
};
use crate::{
    bluetooth::types::BluetoothInfo,
    error::{HyprvisorError, HyprvisorResult},
    hyprland::types::{FormattedInfo, HyprWindowInfo, HyprWorkspaceInfo},
    ipc::{connect_to_socket, message::HyprvisorMessage, HyprvisorReadSock, HyprvisorWriteSock},
    opts::SubscribeOpts,
    wifi::types::WifiInfo,
};

use humantime::format_rfc3339_seconds;
use log::LevelFilter;
use std::{process, time::SystemTime};
use tokio::net::UnixStream;

pub async fn start_client(opts: SubscribeOpts, filter: LevelFilter) -> HyprvisorResult<()> {
    init_logger(filter)?;
    ping_daemon().await?;

    let (subscription_id, extra_data) = parse_opts(opts);
    let stream = subscribe(subscription_id).await?;

    loop {
        let response_message = match stream.try_read_message(3).await {
            Ok(message) => message,
            Err(e) => {
                log::error!("Failed to read message from server: {e}");
                return Err(e);
            }
        };

        println!(
            "{}",
            parse_response(response_message, subscription_id, &extra_data)?
        );
    }
}

fn init_logger(filter: LevelFilter) -> HyprvisorResult<()> {
    let logger = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "({}) {} [{}] {} - {}",
                process::id(),
                format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(filter)
        .chain(fern::log_file("/tmp/hyprvisor-client.log")?);

    let logger = if LevelFilter::Debug == filter {
        logger.chain(std::io::stdout())
    } else {
        logger
    };

    logger
        .apply()
        .map_err(|e| HyprvisorError::LoggerError(fern::InitError::SetLoggerError(e)))
}

fn parse_opts(opts: SubscribeOpts) -> (SubscriptionID, u32) {
    match opts {
        SubscribeOpts::Workspaces { fix_workspace } => (
            SubscriptionID::Workspaces,
            fix_workspace.map_or(0, |fw| {
                log::warn!("Max workspaces = 10");
                fw.min(10)
            }),
        ),
        SubscribeOpts::Window { title_length } => (
            SubscriptionID::Window,
            title_length.map_or(50, |tl| {
                log::warn!("Max title length = 100");
                tl.min(u8::MAX.into())
            }),
        ),
        SubscribeOpts::Wifi { ssid_length } => (
            SubscriptionID::Wifi,
            ssid_length.map_or(25, |sl| sl.min(u8::MAX.into())),
        ),

        SubscribeOpts::Bluetooth => (SubscriptionID::Bluetooth, 0),
    }
}

async fn subscribe(subcription_id: SubscriptionID) -> HyprvisorResult<UnixStream> {
    let stream = connect_to_socket(&HYPRVISOR_SOCKET, 1, 100).await?;

    let message = HyprvisorMessage::from(ClientInfo {
        subscription_id: subcription_id,
        process_id: process::id(),
    });

    stream.try_write_message(&message, 3).await?;

    Ok(stream)
}

fn parse_response(
    message: HyprvisorMessage,
    subcription_id: SubscriptionID,
    extra_data: &u32,
) -> HyprvisorResult<String> {
    match subcription_id {
        SubscriptionID::Workspaces => {
            let ws_info: Vec<HyprWorkspaceInfo> = message.try_into()?;
            ws_info.to_formatted_json(extra_data)
        }
        SubscriptionID::Window => {
            let window_info: HyprWindowInfo = message.try_into()?;
            window_info.to_formatted_json(extra_data)
        }
        SubscriptionID::Wifi => {
            let wifi_info: WifiInfo = message.try_into()?;
            wifi_info.to_formatted_json(extra_data)
        }
        SubscriptionID::Bluetooth => {
            let bt_info: BluetoothInfo = message.try_into()?;
            bt_info.to_formatted_json(extra_data)
        }
        SubscriptionID::Invalid => {
            unreachable!()
        }
    }
}
