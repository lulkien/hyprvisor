use super::types::WifiInfo;
use crate::{
    application::types::SubscriptionID,
    error::{HyprvisorError, HyprvisorResult},
    global::SUBSCRIBERS,
    ipc::{message::HyprvisorMessage, HyprvisorWriteSock},
};

use iwdrs::{modes::Mode, session::Session, station::Station};
use std::{thread::sleep, time::Duration};
use tokio::net::UnixStream;

const POLLING_INTERVAL: u64 = 2000;

pub async fn start_wifi_listener() -> HyprvisorResult<()> {
    let mut current_wifi = WifiInfo::default();

    let session = match Session::new().await {
        Ok(session) => session,
        Err(_) => {
            log::error!("Cannot get iwd session.");
            return Err(HyprvisorError::WifiError);
        }
    };

    let device = match session.device() {
        Some(device) => device,
        None => {
            log::error!("Cannot get iwd device.");
            return Err(HyprvisorError::WifiError);
        }
    };

    if device
        .get_mode()
        .await
        .map_err(|_| HyprvisorError::WifiError)?
        != Mode::Station
    {
        log::error!("Mode not supported.");
        return Err(HyprvisorError::WifiError);
    }

    let station = match session.station() {
        Some(station) => station,
        None => {
            log::error!("Failed to get iwd station");
            return Err(HyprvisorError::WifiError);
        }
    };

    polling_iwd(station, &mut current_wifi).await
}

pub async fn response_to_subscription(stream: &UnixStream) -> HyprvisorResult<()> {
    let init_message: HyprvisorMessage = HyprvisorMessage::try_from(WifiInfo::default())?;
    stream.write_message(init_message).await.map(|_| ())
}

async fn polling_iwd(station: Station, _current_wifi: &mut WifiInfo) -> HyprvisorResult<()> {
    loop {
        match station.state().await {
            Ok(state) => {
                let ssid = if state == "connected" {
                    match station.connected_network().await {
                        Ok(Some(network)) => network
                            .name()
                            .await
                            .unwrap_or_else(|_| "unknown".to_string()),
                        _ => "unknown".to_string(),
                    }
                } else {
                    "unknown".to_string()
                };

                let wifi_info = WifiInfo {
                    state: state.clone(),
                    ssid,
                };

                let _ = broadcast_info(&wifi_info).await;
            }
            Err(_) => {
                log::error!("Cannot get iwd state.");
                return Err(HyprvisorError::WifiError);
            }
        }

        sleep(Duration::from_millis(POLLING_INTERVAL));
    }
}

pub(super) async fn broadcast_info(wifi_info: &WifiInfo) -> HyprvisorResult<()> {
    let mut subscribers_ref = SUBSCRIBERS.lock().await;

    let subscribers = match subscribers_ref.get_mut(&SubscriptionID::Wifi) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            return Err(HyprvisorError::NoSubscriber);
        }
    };

    let message: HyprvisorMessage = HyprvisorMessage::try_from(wifi_info.clone())?;

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
