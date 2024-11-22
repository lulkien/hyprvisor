use super::{types::HyprWindowInfo, CURRENT_WINDOW};
use crate::{
    application::types::SubscriptionID,
    error::{HyprvisorError, HyprvisorResult},
    global::SUBSCRIBERS,
    hyprland::utils::send_hyprland_command,
    ipc::{message::HyprvisorMessage, HyprvisorWriteSock},
};

use tokio::net::UnixStream;

pub async fn response_to_subscription(stream: &UnixStream) -> HyprvisorResult<()> {
    match get_hypr_active_window().await {
        Ok(window) => stream.write_message(window.try_into()?).await.map(|_| ()),
        Err(e) => Err(e),
    }
}

async fn get_hypr_active_window() -> HyprvisorResult<HyprWindowInfo> {
    let json_data: serde_json::Value =
        serde_json::from_slice(&send_hyprland_command("j/activewindow").await?)?;

    Ok(HyprWindowInfo {
        class: json_data["class"].as_str().unwrap_or_default().to_string(),
        title: json_data["title"].as_str().unwrap_or_default().to_string(),
    })
}

pub(super) async fn broadcast_info() -> HyprvisorResult<()> {
    let mut subscribers_ref = SUBSCRIBERS.lock().await;

    let subscribers = match subscribers_ref.get_mut(&SubscriptionID::Window) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            return Err(HyprvisorError::NoSubscriber);
        }
    };

    let current_window = CURRENT_WINDOW.clone();
    let mut current_window = current_window.lock().await;

    let window = get_hypr_active_window().await?;

    if *current_window == window {
        return Err(HyprvisorError::FalseAlarm);
    }

    *current_window = window;

    let message: HyprvisorMessage = HyprvisorMessage::try_from(current_window.clone())?;

    let mut disconnected_pid = Vec::new();

    for (pid, stream) in subscribers.iter_mut() {
        if stream.try_write_message(&message, 2).await.is_err() {
            log::debug!("Client {pid} is disconnected.");
            disconnected_pid.push(*pid);
        }
    }

    for pid in disconnected_pid {
        log::info!("Remove {pid}");
        subscribers.remove(&pid);
    }

    Ok(())
}
