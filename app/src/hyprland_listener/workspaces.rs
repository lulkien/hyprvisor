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
    let ws_regex = Regex::new(
        r"workspace ID \d \((\d)\) on monitor (.+):\n\smonitorID: (\d)\n\swindows: (\d)\n\shasfullscreen: (.+)\n\slastwindow: (.*)\n\slastwindowtitle: (.*)",
    )?;

    if let Some(captures) = ws_regex.captures(raw_data) {
        let id: u32 = captures[1].parse()?;
        let windows: usize = captures[4].parse()?;

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

fn parse_all_workspaces(raw_data: &str) -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
    let mut result_vec = Vec::new();
    for workspace_data in raw_data.split("\n\n") {
        let ws_info = parse_workspace(workspace_data)?;
        log::warn!("{ws_info:?}");
        result_vec.push(ws_info);
    }

    Ok(result_vec)
}

async fn get_hypr_workspace_info() -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
    let cmd_sock = utils::get_hyprland_socket(&HyprSocketType::Command);
    let handle_active_ws = utils::write_to_socket(&cmd_sock, "activeworkspace", 1, 250);
    let handle_all_ws = utils::write_to_socket(&cmd_sock, "workspaces", 1, 250);

    let (active_ws, all_ws) = tokio::try_join!(handle_active_ws, handle_all_ws)?;
    let _active_ws = parse_workspace(&active_ws)?;
    let all_ws = parse_all_workspaces(&all_ws)?;

    Ok(all_ws)
}

pub(super) async fn broadcast_info(
    current_ws_info: &mut Vec<HyprWorkspaceInfo>,
    subscribers: Arc<Mutex<Subscriber>>,
) -> HyprvisorResult<()> {
    let mut subscribers = subscribers.lock().await;
    let _ws_subscribers = match subscribers.get_mut(&SubscriptionID::Workspaces) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            log::info!("No subscribers");
            return Ok(());
        }
    };

    let new_ws_info = get_hypr_workspace_info().await?;
    if *current_ws_info == new_ws_info {
        log::warn!("False Alarm");
        return Ok(());
    }

    *current_ws_info = new_ws_info;

    Ok(())
}
