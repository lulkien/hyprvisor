use serde::{Deserialize, Serialize};

use crate::{
    error::{HyprvisorError, HyprvisorResult},
    hyprland::types::FormattedInfo,
    ipc::message::HyprvisorMessage,
};

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WifiInfo {
    pub state: String,
    pub ssid: String,
}

impl Default for WifiInfo {
    fn default() -> Self {
        Self {
            state: "loading".to_string(),
            ssid: "Identifying...".to_string(),
        }
    }
}

impl FormattedInfo for WifiInfo {
    fn to_formatted_json(mut self, extra_data: &u32) -> HyprvisorResult<String> {
        if let Some(title) = self.ssid.get(..*extra_data as usize) {
            self.ssid = format!("{}...", String::from_utf8_lossy(title.as_bytes()));
        }

        serde_json::to_string(&self).map_err(HyprvisorError::JsonError)
    }
}

impl TryFrom<HyprvisorMessage> for WifiInfo {
    type Error = HyprvisorError;
    fn try_from(message: HyprvisorMessage) -> HyprvisorResult<WifiInfo> {
        if !message.is_valid() {
            return Err(HyprvisorError::InvalidMessage);
        }
        bincode::deserialize(&message.payload).map_err(HyprvisorError::BincodeError)
    }
}
