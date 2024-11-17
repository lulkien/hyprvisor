use super::{types::HyprWorkspaceInfo, utils::send_hyprland_command};
use crate::{
    application::types::SubscriptionID,
    error::{HyprvisorError, HyprvisorResult},
    global::SUBSCRIBERS,
    ipc::{message::HyprvisorMessage, HyprvisorWriteSock},
};

use serde_json::{from_slice, Value};
use tokio::net::UnixStream;

pub async fn response_to_subscription(stream: &UnixStream) -> HyprvisorResult<()> {
    match get_hypr_workspace_info().await {
        Ok(ws_info) => stream.write_message(ws_info.try_into()?).await.map(|_| ()),
        Err(e) => Err(e),
    }
}

async fn get_hypr_workspace_info() -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
    let (active_workspace, all_workspace) = tokio::try_join!(
        send_hyprland_command("j/activeworkspace"),
        send_hyprland_command("j/workspaces")
    )?;

    let active_ws_id = from_slice::<Value>(&active_workspace)?["id"]
        .as_u64()
        .unwrap_or_default() as u32;

    match from_slice(&all_workspace)? {
        Value::Array(json_array) => Ok(json_array
            .iter()
            .map(|js_obj| HyprWorkspaceInfo {
                id: js_obj["id"].as_u64().unwrap_or_default() as u32,
                occupied: js_obj["windows"].as_i64().unwrap_or_default() > 0,
                active: js_obj["id"].as_u64().unwrap_or_default() as u32 == active_ws_id,
            })
            .collect()),
        _ => Err(HyprvisorError::ParseError),
    }
}

pub(super) async fn broadcast_info(
    current_ws_info: &mut Vec<HyprWorkspaceInfo>,
) -> HyprvisorResult<()> {
    let mut subscribers_ref = SUBSCRIBERS.lock().await;

    let ws_subscribers = match subscribers_ref.get_mut(&SubscriptionID::Workspaces) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            return Err(HyprvisorError::NoSubscriber);
        }
    };

    let new_ws_info = get_hypr_workspace_info().await?;
    if *current_ws_info == new_ws_info {
        return Err(HyprvisorError::FalseAlarm);
    }

    *current_ws_info = new_ws_info.clone();
    let message: HyprvisorMessage = HyprvisorMessage::try_from(new_ws_info)?;

    let mut disconnected_pid = Vec::new();

    for (pid, stream) in ws_subscribers.iter_mut() {
        if stream.try_write_message(&message, 2).await.is_err() {
            log::debug!("Client {pid} is disconnected.");
            disconnected_pid.push(*pid);
        }
    }

    for pid in disconnected_pid {
        log::info!("Remove pid: {pid}");
        ws_subscribers.remove(&pid);
    }

    Ok(())
}