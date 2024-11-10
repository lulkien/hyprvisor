use crate::{
    common_types::{ClientInfo, SubscriptionID},
    error::{HyprvisorError, HyprvisorResult},
    hyprland::types::{HyprWinInfo, HyprWorkspaceInfo},
    ipc::{connect_to_socket, HyprvisorSocket},
    opts::{CommandOpts, SubscribeOpts},
};
use std::{collections::HashMap, process};
use tokio::io::AsyncReadExt;

pub(crate) async fn start_client(
    socket: &str,
    subscription_opts: &SubscribeOpts,
) -> HyprvisorResult<()> {
    let (sub_id, data_format): (SubscriptionID, u32) = match subscription_opts {
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
        SubscribeOpts::Wireless => {
            todo!()
        }
    };

    let pid = process::id();
    let client_info = ClientInfo::new(pid, sub_id.clone());
    let subscribe_msg = serde_json::to_string(&client_info)?;

    let mut stream = connect_to_socket(socket, 5, 500).await?;
    let init_response = stream.write_and_read_multiple(&subscribe_msg, 10).await?;
    let result = reformat_response(init_response.as_slice(), &sub_id, &data_format)?;

    println!("{result}");

    loop {
        let mut buffer: [u8; 1024] = [0; 1024];
        let bytes_received = match stream.read(&mut buffer).await {
            Ok(size) if size > 0 => size,
            Ok(_) | Err(_) => {
                log::error!("Error reading from server.");
                return Err(HyprvisorError::StreamError);
            }
        };

        let result = reformat_response(&buffer[..bytes_received], &sub_id, &data_format)?;
        println!("{result}");
    }
}

pub(crate) async fn send_server_command(
    socket_path: &str,
    command: &CommandOpts,
    max_attempts: u8,
) -> HyprvisorResult<()> {
    let stream = connect_to_socket(socket_path, max_attempts, 200).await?;
    let message = serde_json::to_string(&command)?;

    stream.write_once(&message).await?;
    let response = stream.read_once().await?;

    log::info!(
        "Response from server: {}",
        String::from_utf8(response).unwrap()
    );
    Ok(())
}

fn reformat_response(
    buffer: &[u8],
    subscription_id: &SubscriptionID,
    extra_data: &u32,
) -> HyprvisorResult<String> {
    let mut formatted_response = String::from_utf8_lossy(buffer).to_string();
    if *extra_data < 1 {
        return Ok(formatted_response);
    }

    match subscription_id {
        SubscriptionID::Workspaces => {
            let origin: Vec<HyprWorkspaceInfo> = serde_json::from_str(&formatted_response)?;
            if origin.len() > *extra_data as usize {
                return Ok(formatted_response);
            }

            let mut table: HashMap<u32, HyprWorkspaceInfo> =
                origin.clone().into_iter().map(|ws| (ws.id, ws)).collect();

            (1..=*extra_data).for_each(|id| {
                table.entry(id).or_insert_with(|| HyprWorkspaceInfo {
                    id,
                    occupied: false,
                    active: false,
                });
            });

            let mut modified: Vec<HyprWorkspaceInfo> = table.into_values().collect();
            modified.sort_by_key(|info| info.id);

            formatted_response = serde_json::to_string(&modified)?;
        }
        SubscriptionID::Window => {
            let win_info: Result<HyprWinInfo, _> = serde_json::from_slice(buffer);
            let mut win_info = match win_info {
                Ok(value) => value,
                Err(_) => return Ok(formatted_response),
            };

            if let Some(title) = win_info.title.get(..*extra_data as usize) {
                win_info.title = format!("{}...", String::from_utf8_lossy(title.as_bytes()));
                formatted_response = serde_json::to_string(&win_info)?;
            }
        }
        SubscriptionID::Wireless => {}
    }

    Ok(formatted_response)
}
