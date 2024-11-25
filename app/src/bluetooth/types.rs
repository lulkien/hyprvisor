use crate::{
    error::{HyprvisorError, HyprvisorResult},
    hyprland::types::FormattedInfo,
    ipc::message::{HyprvisorMessage, MessageType},
};

use bluer::Address;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct BluetoothDeviceInfo {
    pub(super) name: String,
    pub(super) address: Address,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(super) struct BluetoothInfo {
    pub power_state: bool,
    pub connected_device_len: u16,
    pub connected_device: Vec<BluetoothDeviceInfo>,
}

impl FormattedInfo for BluetoothInfo {
    fn to_formatted_json(self, _extra_data: &u32) -> HyprvisorResult<String> {
        serde_json::to_string(&self).map_err(HyprvisorError::JsonError)
    }
}

impl TryFrom<HyprvisorMessage> for BluetoothInfo {
    type Error = HyprvisorError;
    fn try_from(message: HyprvisorMessage) -> HyprvisorResult<BluetoothInfo> {
        if !message.is_valid() {
            return Err(HyprvisorError::InvalidMessage);
        }
        bincode::deserialize(&message.payload).map_err(HyprvisorError::BincodeError)
    }
}

impl TryFrom<BluetoothInfo> for HyprvisorMessage {
    type Error = HyprvisorError;
    fn try_from(wifi_info: BluetoothInfo) -> Result<Self, Self::Error> {
        let payload: Vec<u8> = bincode::serialize(&wifi_info)?;
        Ok(HyprvisorMessage {
            message_type: MessageType::Response,
            header: payload.len(),
            payload,
        })
    }
}
