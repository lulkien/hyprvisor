use super::types::{HyprSocketType, HyprWinInfo};
use crate::{
    common_types::{Subscriber, SubscriptionID},
    error::{HyprvisorError, HyprvisorResult},
    utils,
};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, sync::Mutex};

pub(crate) async fn get_hypr_active_window() -> HyprvisorResult<HyprWinInfo> {
    use serde_json::{from_str, Value};

    let cmd_sock = utils::get_hyprland_socket(&HyprSocketType::Command);
    let raw_response = utils::write_to_socket(&cmd_sock, "j/activewindow", 1, 250).await?;
    let json_data: Value = from_str(&raw_response)?;

    Ok(HyprWinInfo {
        class: json_data["class"].as_str().unwrap_or_default().to_string(),
        title: json_data["title"].as_str().unwrap_or_default().to_string(),
    })
}

pub(super) async fn broadcast_info(
    current_win_info: &mut HyprWinInfo,
    subscribers: Arc<Mutex<Subscriber>>,
) -> HyprvisorResult<()> {
    let mut subscribers = subscribers.lock().await;
    let win_subscribers = match subscribers.get_mut(&SubscriptionID::Window) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            return Err(HyprvisorError::NoSubscriber);
        }
    };

    let new_win_info = get_hypr_active_window().await?;
    if *current_win_info == new_win_info {
        return Err(HyprvisorError::FalseAlarm);
    }

    *current_win_info = new_win_info.clone();

    let mut disconnected_pid = Vec::new();
    let window_json = serde_json::to_string(current_win_info)?;

    for (pid, stream) in win_subscribers.iter_mut() {
        if stream.write_all(window_json.as_bytes()).await.is_err() {
            println!("Client {} is disconnected. Remove later", pid);
            disconnected_pid.push(*pid);
        }
    }

    for pid in disconnected_pid {
        win_subscribers.remove(&pid);
    }

    Ok(())
}
