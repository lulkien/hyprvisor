use serde::{Deserialize, Serialize};

use crate::{
    error::{HyprvisorError, HyprvisorResult},
    hyprland::types::FormattedInfo,
    ipc::message::HyprvisorMessage,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WifiState {
    Disabled,
    Disconnected,
    Connecting,
    Connected,
    Unknown,
}

impl Default for WifiState {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<&str> for WifiState {
    fn from(value: &str) -> Self {
        match value {
            "disabled" => Self::Disabled,
            "disconnected" => Self::Disconnected,
            "connecting" => Self::Connecting,
            "connected" => Self::Connected,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct WifiInfo {
    pub state: WifiState,
    pub ssid: String,
    pub icon: String,
}

impl WifiInfo {
    pub fn get_wifi_icon(state: WifiState) -> String {
        match state {
            WifiState::Unknown => "󱚵",
            WifiState::Disabled => "󰖪",
            WifiState::Connected => "󰖩",
            WifiState::Connecting => "󱛇",
            WifiState::Disconnected => "󱛅",
        }
        .to_string()
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
