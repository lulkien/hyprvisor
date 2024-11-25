use super::MAX_ATTEMPT_RETRY;
use crate::{bluetooth::REBOOT_IWD_DELAY, error::HyprvisorResult};

use std::{thread::sleep, time::Duration};

pub async fn start_bluetooth_listener() -> HyprvisorResult<()> {
    for attempt in 0..MAX_ATTEMPT_RETRY {
        log::info!(
            "Attemp to start bluetooth listener: {}/{}",
            attempt + 1,
            MAX_ATTEMPT_RETRY
        );

        log::warn!("Bluetooth is down. Rebooting...");
        sleep(Duration::from_millis(REBOOT_IWD_DELAY));
    }

    Ok(())
}
