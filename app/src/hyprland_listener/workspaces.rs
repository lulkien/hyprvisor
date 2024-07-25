use super::types::{HyprSocketType, HyprWorkspaceInfo};
use crate::{
    common_types::{Subscriber, SubscriptionID},
    error::{HyprvisorError, HyprvisorResult},
    utils,
};
use std::sync::Arc;
use tokio::sync::Mutex;

fn parse_all_workspaces(
    raw_data: &str,
    raw_active: &str,
) -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
    use serde_json::{from_str, Value};

    let json_objects: Value = from_str(raw_data)?;
    let active_id = get_activeworkspace_id(raw_active)?;
    if let Value::Array(js_arr) = json_objects {
        let result_vec: Vec<HyprWorkspaceInfo> = js_arr
            .iter()
            .map(|js_obj| HyprWorkspaceInfo {
                id: js_obj["id"].as_u64().unwrap_or_default() as u32,
                occupied: js_obj["windows"].as_i64().unwrap_or_default() > 0,
                active: js_obj["id"].as_u64().unwrap_or_default() as u32 == active_id,
            })
            .collect();

        return Ok(result_vec);
    }
    Err(HyprvisorError::ParseError)
}

fn get_activeworkspace_id(raw_data: &str) -> HyprvisorResult<u32> {
    use serde_json::{from_str, Value};
    let json_objects: Value = from_str(raw_data)?;
    Ok(json_objects["id"].as_u64().unwrap_or_default() as u32)
}

pub(crate) async fn get_hypr_workspace_info() -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
    let cmd_sock = utils::get_hyprland_socket(&HyprSocketType::Command);

    let handle_active_ws = utils::try_connect_and_wait(&cmd_sock, "j/activeworkspace", 10);
    let handle_all_ws = utils::try_connect_and_wait(&cmd_sock, "j/workspaces", 10);

    let (raw_active, raw_all) = tokio::try_join!(handle_active_ws, handle_all_ws)?;

    let all_ws = parse_all_workspaces(&raw_all, &raw_active)?;

    Ok(all_ws)
}

pub(super) async fn broadcast_info(
    current_ws_info: &mut Vec<HyprWorkspaceInfo>,
    subscribers: Arc<Mutex<Subscriber>>,
) -> HyprvisorResult<()> {
    let mut subscribers = subscribers.lock().await;
    let ws_subscribers = match subscribers.get_mut(&SubscriptionID::Workspaces) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            return Err(HyprvisorError::NoSubscriber);
        }
    };

    let new_ws_info = get_hypr_workspace_info().await?;
    if *current_ws_info == new_ws_info {
        return Err(HyprvisorError::FalseAlarm);
    }

    *current_ws_info = new_ws_info;

    let mut disconnected_pid = Vec::new();
    let ws_json = serde_json::to_string(current_ws_info)?;

    for (pid, stream) in ws_subscribers.iter_mut() {
        if utils::try_write(stream, &ws_json).await.is_err() {
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
