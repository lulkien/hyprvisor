use super::FormattedInfo;
use crate::{
    error::{HyprvisorError, HyprvisorResult},
    ipc::message::HyprvisorMessage,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct HyprWindowInfo {
    pub class: String,
    pub title: String,
}

impl FormattedInfo for HyprWindowInfo {
    fn to_formatted_json(mut self, extra_data: &u32) -> HyprvisorResult<String> {
        if let Some(title) = self.title.get(..*extra_data as usize) {
            self.title = format!("{}...", String::from_utf8_lossy(title.as_bytes()));
        }

        serde_json::to_string(&self).map_err(HyprvisorError::JsonError)
    }
}

impl TryFrom<HyprvisorMessage> for HyprWindowInfo {
    type Error = HyprvisorError;
    fn try_from(message: HyprvisorMessage) -> HyprvisorResult<HyprWindowInfo> {
        if !message.is_valid() {
            return Err(HyprvisorError::InvalidMessage);
        }
        bincode::deserialize(&message.payload).map_err(HyprvisorError::BincodeError)
    }
}
