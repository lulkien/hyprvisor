use super::types::{HyprSocketType, HyprWorkspaceInfo};
use crate::{
    common_types::{Subscriber, SubscriptionID},
    error::{HyprvisorError, HyprvisorResult},
    utils,
};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, sync::Mutex};

async fn get_hypr_workspace_info() -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
    let cmd_sock = utils::get_hyprland_socket(&HyprSocketType::Command);
    let handle_active_ws = utils::write_to_socket(&cmd_sock, "activeworkspace", 1, 250);
    let handle_all_ws = utils::write_to_socket(&cmd_sock, "workspaces", 1, 250);

    tokio::try_join!(handle_active_ws, handle_all_ws)?;

    Ok(vec![])
}

pub(super) async fn broadcast_info(
    current_ws_info: &mut [HyprWorkspaceInfo],
    subscribers: Arc<Mutex<Subscriber>>,
) -> HyprvisorResult<()> {
    let mut subscribers = subscribers.lock().await;
    let ws_subscribers = match subscribers.get_mut(&SubscriptionID::Window) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            log::info!("No subscribers");
            return Err(HyprvisorError::NoSubscribers);
        }
    };

    let new_ws_info = get_hypr_workspace_info().await?;
    if *current_ws_info == new_ws_info {
        return Err(HyprvisorError::FalseAlarm);
    }

    log::debug!("Hello");
    Ok(())
}
