use super::FormattedInfo;
use crate::{
    error::{HyprvisorError, HyprvisorResult},
    ipc::message::{HyprvisorMessage, MessageType},
};

use bincode::{
    config::legacy,
    serde::{decode_from_slice, encode_to_vec},
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

        let decoded: Result<(HyprWindowInfo, usize), _> =
            decode_from_slice(&message.payload, legacy());

        match decoded {
            Ok((bt, len)) if len == message.payload.len() => Ok(bt),
            Err(_) | Ok(_) => Err(HyprvisorError::BincodeError),
        }
    }
}

impl TryFrom<HyprWindowInfo> for HyprvisorMessage {
    type Error = HyprvisorError;
    fn try_from(window_info: HyprWindowInfo) -> HyprvisorResult<HyprvisorMessage> {
        let payload: Vec<u8> = encode_to_vec(window_info, legacy())?;

        Ok(HyprvisorMessage {
            message_type: MessageType::Response,
            header: payload.len(),
            payload,
        })
    }
}
