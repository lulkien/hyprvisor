use super::{
    types::{WifiInfo, WifiState},
    CURRENT_WIFI, MAX_ATTEMPT_RETRY, POLLING_INTERVAL,
};
use crate::{
    application::types::SubscriptionID,
    error::{HyprvisorError, HyprvisorResult},
    global::SUBSCRIBERS,
    ipc::{message::HyprvisorMessage, HyprvisorWriteSock},
    wifi::REBOOT_IWD_DELAY,
};

use iwdrs::{modes::Mode, session::Session, station::Station};
use std::{thread::sleep, time::Duration};
use tokio::net::UnixStream;

pub async fn start_wifi_listener() -> HyprvisorResult<()> {
    log::debug!("Start wifi listener");

    for attempt in 0..MAX_ATTEMPT_RETRY {
        log::info!(
            "Attemp to start wifi listener: {}/{}",
            attempt + 1,
            MAX_ATTEMPT_RETRY
        );
        let _ = connect_to_iwd_session().await;

        log::warn!("Iwd is down. Rebooting...");
        sleep(Duration::from_millis(REBOOT_IWD_DELAY));
    }

    log::error!("Cannot start wifi listener. Out of attempt.");
    Err(HyprvisorError::WifiError)
}

pub async fn response_to_subscription(stream: &UnixStream) -> HyprvisorResult<()> {
    let current_wifi = CURRENT_WIFI.lock().await;

    let init_message: HyprvisorMessage = HyprvisorMessage::try_from((*current_wifi).clone())?;
    stream.write_message(init_message).await.map(|_| ())
}

async fn connect_to_iwd_session() -> HyprvisorResult<()> {
    let session = match Session::new().await {
        Ok(session) => session,
        Err(_) => {
            log::error!("Failed to create a new iwd session.");
            return Err(HyprvisorError::WifiError);
        }
    };

    let device = match session.device() {
        Some(device) => device,
        None => {
            log::error!("No iwd device found.");
            return Err(HyprvisorError::WifiError);
        }
    };

    match device.get_mode().await {
        Ok(Mode::Station) => {
            log::debug!("Device is in Station mode.");
        }
        Ok(mode) => {
            log::warn!("Device is not in Station mode. Current mode: {mode}");
            return Err(HyprvisorError::WifiError);
        }
        Err(_) => {
            log::error!("Failed to retrieve the device mode.");
            return Err(HyprvisorError::WifiError);
        }
    };

    let station = match session.station() {
        Some(station) => station,
        None => {
            log::error!("Failed to retrieve the iwd station.");
            return Err(HyprvisorError::WifiError);
        }
    };

    polling_data(station).await
}

async fn polling_data(station: Station) -> HyprvisorResult<()> {
    loop {
        let wifi_info = match station.state().await {
            Ok(state) => {
                let mut ssid = String::new();

                if state == "connected" {
                    ssid = match station.connected_network().await {
                        Ok(Some(network)) => network.name().await.unwrap_or_else(|_| String::new()),
                        _ => String::new(),
                    }
                };

                let wifi_state = WifiState::from(state.as_str());

                WifiInfo {
                    state: wifi_state.clone(),
                    ssid,
                    icon: WifiInfo::get_wifi_icon(wifi_state),
                }
            }
            Err(_) => {
                log::error!("Cannot get iwd state.");
                WifiInfo {
                    state: WifiState::Disabled,
                    ssid: String::new(),
                    icon: WifiInfo::get_wifi_icon(WifiState::Disabled),
                }
            }
        };

        handle_wifi_info(wifi_info).await?;

        sleep(Duration::from_millis(POLLING_INTERVAL));
    }
}

async fn handle_wifi_info(wifi_info: WifiInfo) -> HyprvisorResult<()> {
    let mut current_wifi = CURRENT_WIFI.lock().await;

    if *current_wifi == wifi_info {
        return Ok(());
    }

    *current_wifi = wifi_info.clone();

    let _ = broadcast_info(wifi_info).await;

    if current_wifi.state == WifiState::Disabled {
        return Err(HyprvisorError::WifiError);
    }

    Ok(())
}

async fn broadcast_info(wifi_info: WifiInfo) -> HyprvisorResult<()> {
    let mut subscribers_ref = SUBSCRIBERS.lock().await;

    let subscribers = match subscribers_ref.get_mut(&SubscriptionID::Wifi) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            return Err(HyprvisorError::NoSubscriber);
        }
    };

    let message: HyprvisorMessage = HyprvisorMessage::try_from(wifi_info)?;

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
