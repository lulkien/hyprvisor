use super::types::{HyprSocketType, HyprWorkspaceInfo};
use crate::{
    common_types::{Subscriber, SubscriptionID},
    error::{HyprvisorError, HyprvisorResult},
    utils,
};
use regex::Regex;
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, sync::Mutex};

fn parse_workspace(raw_data: &str) -> HyprvisorResult<HyprWorkspaceInfo> {
    lazy_static! {
        static ref WS_REGEX: Regex = Regex::new(
            r"workspace ID \d+ \((\d+)\) on monitor.*:\n\s*monitorID:.*\n\s*windows: (\d+)\n\s*hasfullscreen:.*\n\s*lastwindow:.*\n\s*lastwindowtitle:.*",
        ).expect("Failed to compile regex");
    }

    if let Some(captures) = WS_REGEX.captures(raw_data) {
        let id: u32 = captures[1].parse()?;
        let windows: usize = captures[2].parse()?;

        let workspace_info = HyprWorkspaceInfo {
            id,
            occupied: windows != 0,
            active: false,
        };

        Ok(workspace_info)
    } else {
        Err(HyprvisorError::ParseError)
    }
}

fn parse_all_workspaces(
    raw_data: &str,
    active_ws: HyprWorkspaceInfo,
) -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
    let result_vec: Vec<HyprWorkspaceInfo> = raw_data
        .split("\n\n")
        .filter_map(|workspace_data| {
            let trimmed_data = workspace_data.trim();
            if trimmed_data.is_empty() {
                None
            } else {
                let mut ws_info = parse_workspace(trimmed_data).ok()?;
                if ws_info.id == active_ws.id {
                    ws_info.active = true;
                }
                Some(ws_info)
            }
        })
        .collect();

    Ok(result_vec)
}

pub(crate) async fn get_hypr_workspace_info() -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
    let cmd_sock = utils::get_hyprland_socket(&HyprSocketType::Command);

    let handle_active_ws = utils::write_to_socket(&cmd_sock, "activeworkspace", 1, 250);
    let handle_all_ws = utils::write_to_socket(&cmd_sock, "workspaces", 1, 250);

    let (active_ws, all_ws) = tokio::try_join!(handle_active_ws, handle_all_ws)?;

    let mut active_ws = parse_workspace(&active_ws)?;
    active_ws.active = true;

    let all_ws = parse_all_workspaces(&all_ws, active_ws)?;

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
        if stream.write_all(ws_json.as_bytes()).await.is_err() {
            log::info!("Client {} is disconnected. Remove later", pid);
            disconnected_pid.push(*pid);
        }
    }

    for pid in disconnected_pid {
        ws_subscribers.remove(&pid);
    }

    Ok(())
}
