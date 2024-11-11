use super::{utils::ping_daemon, utils::HYPRVISOR_SOCKET};
use crate::{
    error::{HyprvisorError, HyprvisorResult},
    global::BUFFER_SIZE,
    hyprland::types::{HyprWinInfo, HyprWorkspaceInfo},
    ipc::{connect_to_socket, HyprvisorSocket},
    opts::SubscribeOpts,
    types::{ClientInfo, SubscriptionID},
};

use humantime::format_rfc3339_seconds;
use log::LevelFilter;
use std::{collections::HashMap, process, time::SystemTime};
use tokio::net::UnixStream;

pub async fn start_client(
    subscription_opts: &SubscribeOpts,
    filter: LevelFilter,
) -> HyprvisorResult<()> {
    init_logger(filter)?;
    ping_daemon().await?;

    let (subcription_id, extra_data): (SubscriptionID, u32) = match subscription_opts {
        SubscribeOpts::Workspaces { fix_workspace } => (
            SubscriptionID::Workspaces,
            fix_workspace.map_or(0, |fw| {
                log::warn!("Max workspaces = 10");
                fw.min(10)
            }),
        ),
        SubscribeOpts::Window { title_length } => (
            SubscriptionID::Window,
            title_length.map_or(0, |tl| {
                log::warn!("Max title length = 100");
                tl.min(100)
            }),
        ),
        SubscribeOpts::Wireless { max_ssid_length: _ } => {
            todo!()
        }
    };

    let stream = subscribe(&subcription_id).await?;

    let mut buffer = vec![0u8; *BUFFER_SIZE];
    loop {
        let byte_len = match stream.read_multiple_buf(&mut buffer, 3).await {
            Ok(len) => len,
            Err(e) => {
                log::error!("Error reading from server: {e}");
                return Err(e);
            }
        };

        println!(
            "{}",
            reformat_response(&buffer[..byte_len], &subcription_id, &extra_data)?
        );
    }
}

fn init_logger(filter: LevelFilter) -> HyprvisorResult<()> {
    let logger = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "({}){} [{}] {} - {}",
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

    logger.apply().map_err(|_| HyprvisorError::LoggerError)
}

async fn subscribe(subcription_id: &SubscriptionID) -> HyprvisorResult<UnixStream> {
    let stream = connect_to_socket(&HYPRVISOR_SOCKET, 1, 100).await?;

    let subcription_msg =
        serde_json::to_string(&ClientInfo::new(process::id(), subcription_id.clone()))?;

    stream.write_multiple(&subcription_msg, 3).await?;

    Ok(stream)
}

fn reformat_response(
    buffer: &[u8],
    subscription_id: &SubscriptionID,
    extra_data: &u32,
) -> HyprvisorResult<String> {
    if *extra_data < 1 {
        return Ok(String::from_utf8_lossy(buffer).to_string());
    }

    match subscription_id {
        SubscriptionID::Workspaces => reformat_workspace_response(buffer, extra_data),
        SubscriptionID::Window => reformat_window_response(buffer, extra_data),
        SubscriptionID::Wireless => reformat_wireless_response(buffer, extra_data),
    }
}

fn reformat_workspace_response(buffer: &[u8], extra_data: &u32) -> HyprvisorResult<String> {
    let workspaces: Vec<HyprWorkspaceInfo> = serde_json::from_slice(buffer)?;
    if workspaces.len() > *extra_data as usize {
        return Ok(String::from_utf8_lossy(buffer).to_string());
    }

    let mut table: HashMap<u32, HyprWorkspaceInfo> =
        workspaces.into_iter().map(|ws| (ws.id, ws)).collect();

    (1..=*extra_data).for_each(|id| {
        table.entry(id).or_insert_with(|| HyprWorkspaceInfo {
            id,
            occupied: false,
            active: false,
        });
    });

    let mut modified: Vec<HyprWorkspaceInfo> = table.into_values().collect();
    modified.sort_by_key(|info| info.id);

    serde_json::to_string(&modified).map_err(|_| HyprvisorError::ParseError)
}

fn reformat_window_response(buffer: &[u8], extra_data: &u32) -> HyprvisorResult<String> {
    let mut win_info: HyprWinInfo = match serde_json::from_slice(buffer) {
        Ok(value) => value,
        Err(_) => return Ok(String::from_utf8_lossy(buffer).to_string()),
    };

    if let Some(title) = win_info.title.get(..*extra_data as usize) {
        win_info.title = format!("{}...", String::from_utf8_lossy(title.as_bytes()));
    }

    serde_json::to_string(&win_info).map_err(|_| HyprvisorError::ParseError)
}

fn reformat_wireless_response(_buffer: &[u8], _extra_data: &u32) -> HyprvisorResult<String> {
    todo!()
}
