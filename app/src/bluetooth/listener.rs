use super::{
    types::{BluetoothDeviceInfo, BluetoothInfo},
    BLUETOOTH_DEVICES, BLUETOOTH_POWERED, MAX_ATTEMPT_RETRY, POLLING_INTERVAL,
};
use crate::{
    application::types::SubscriptionID,
    bluetooth::REBOOT_IWD_DELAY,
    error::{HyprvisorError, HyprvisorResult},
    global::SUBSCRIBERS,
    ipc::{message::HyprvisorMessage, HyprvisorWriteSock},
};

use bluer::{Adapter, Address, Session};
use std::{sync::atomic::Ordering, time::Duration};
use tokio::{net::UnixStream, time::sleep};

pub async fn start_bluetooth_listener() -> HyprvisorResult<()> {
    for attempt in 0..MAX_ATTEMPT_RETRY {
        log::info!(
            "Attemp to start bluetooth listener: {}/{}",
            attempt + 1,
            MAX_ATTEMPT_RETRY
        );

        let _ = connect_to_bluetooth_session().await;

        log::warn!("Bluetooth is down. Rebooting...");
        sleep(Duration::from_millis(REBOOT_IWD_DELAY)).await;
    }

    Ok(())
}

pub async fn response_to_subscription(stream: &UnixStream) -> HyprvisorResult<()> {
    let bt_info = match !BLUETOOTH_POWERED.load(Ordering::SeqCst) {
        true => BluetoothInfo {
            powered: true,
            connected_device: (BLUETOOTH_DEVICES.lock().await).clone(),
        },
        false => BluetoothInfo::default(),
    };

    log::debug!("Bluetooth info: {}", serde_json::to_string(&bt_info)?);

    stream
        .write_message(HyprvisorMessage::try_from(bt_info)?)
        .await
        .map(|_| ())
}

async fn connect_to_bluetooth_session() -> HyprvisorResult<()> {
    let session = match Session::new().await {
        Ok(session) => session,
        Err(_) => {
            log::error!("Failed to connect to bluetooth session.");
            return Err(HyprvisorError::BluetoothError);
        }
    };

    let adapter = match session.default_adapter().await {
        Ok(adapter) => adapter,
        Err(_) => {
            log::error!("Failed to get default adapter");
            return Err(HyprvisorError::BluetoothError);
        }
    };

    let discovered_addresses = match adapter.device_addresses().await {
        Ok(addresses) => addresses,
        Err(_) => {
            log::error!("Failed to get discovered addresses");
            return Err(HyprvisorError::BluetoothError);
        }
    };

    polling_data(adapter, discovered_addresses).await
}

async fn polling_data(adapter: Adapter, discovered_addresses: Vec<Address>) -> HyprvisorResult<()> {
    loop {
        sleep(Duration::from_millis(POLLING_INTERVAL)).await;

        let powered = adapter
            .is_powered()
            .await
            .map_err(|_| HyprvisorError::BluetoothError)?;

        if !handle_power_state(powered).await {
            continue;
        }

        let mut connected_devices: Vec<BluetoothDeviceInfo> = Vec::new();

        for addr in discovered_addresses.iter() {
            if let Ok(device) = adapter.device(*addr) {
                if device.is_connected().await.unwrap_or(false) {
                    connected_devices.push(BluetoothDeviceInfo {
                        name: device
                            .name()
                            .await
                            .unwrap_or(None)
                            .unwrap_or("Unknown device".to_string()),
                        address: *addr,
                    });
                }
            }
        }

        handle_connected_devices(connected_devices).await;
    }
}

async fn handle_power_state(powered: bool) -> bool {
    if powered != BLUETOOTH_POWERED.load(Ordering::SeqCst) {
        BLUETOOTH_POWERED.store(powered, Ordering::SeqCst);
        if !powered {
            let _ = broadcast_info(BluetoothInfo::default()).await;
        }
    }

    powered
}

async fn handle_connected_devices(connected_devices: Vec<BluetoothDeviceInfo>) {
    let mut current_devices = BLUETOOTH_DEVICES.lock().await;

    if *current_devices == connected_devices {
        return;
    }

    *current_devices = connected_devices;
    let _ = broadcast_info(BluetoothInfo {
        powered: true,
        connected_device: (*current_devices).clone(),
    })
    .await;
}

async fn broadcast_info(bluetooth_info: BluetoothInfo) -> HyprvisorResult<()> {
    let mut subscribers_ref = SUBSCRIBERS.lock().await;

    let subscribers = match subscribers_ref.get_mut(&SubscriptionID::Bluetooth) {
        Some(subs) if !subs.is_empty() => subs,
        Some(_) | None => {
            return Err(HyprvisorError::NoSubscriber);
        }
    };

    let message: HyprvisorMessage = HyprvisorMessage::try_from(bluetooth_info)?;

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
