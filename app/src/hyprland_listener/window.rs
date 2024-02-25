use super::types::{HyprSocketType, HyprWinInfo};
use crate::{
    common_types::{Subscriber, SubscriptionID},
    error::HyprvisorResult,
    utils,
};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, sync::Mutex};

async fn get_hypr_active_window() -> HyprvisorResult<HyprWinInfo> {
    let mut win_info = HyprWinInfo::default();
    let cmd_sock = utils::get_hyprland_socket(&HyprSocketType::Command);
    let raw_response = utils::write_to_socket(&cmd_sock, "activewindow", 1, 250).await?;

    let processed_data: Vec<String> = raw_response
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| s.starts_with("class: ") || s.starts_with("title: "))
        .collect();

    for d in processed_data {
        if d.starts_with("class: ") {
            win_info.class = d.strip_prefix("class: ").unwrap().to_string();
        } else if d.starts_with("title: ") {
            win_info.title = d.strip_prefix("title: ").unwrap().to_string();
        } else {
            // do nothing
        }
    }

    Ok(win_info)
}

pub(super) async fn broadcast_info(
    current_win_info: &mut HyprWinInfo,
    subscribers: Arc<Mutex<Subscriber>>,
) -> HyprvisorResult<()> {
    let mut subscribers = subscribers.lock().await;
    let win_subscribers = match subscribers.get_mut(&SubscriptionID::Window) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            log::info!("No subscribers");
            return Ok(());
        }
    };

    let new_win_info = get_hypr_active_window().await?;
    if *current_win_info == new_win_info {
        log::warn!("False Alarm");
        return Ok(());
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
