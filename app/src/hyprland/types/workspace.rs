use super::FormattedInfo;
use crate::{
    error::{HyprvisorError, HyprvisorResult},
    ipc::message::{HyprvisorMessage, MessageType},
};

use bincode::{config::legacy, serde::encode_to_vec};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct HyprWorkspaceInfo {
    pub id: u32,
    pub occupied: bool,
    pub active: bool,
}

impl HyprWorkspaceInfo {
    pub fn default_workspace(id: u32) -> Self {
        Self {
            id,
            occupied: false,
            active: false,
        }
    }
}

impl TryFrom<HyprvisorMessage> for Vec<HyprWorkspaceInfo> {
    type Error = HyprvisorError;
    fn try_from(message: HyprvisorMessage) -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
        if !message.is_valid() {
            return Err(HyprvisorError::InvalidMessage);
        }

        let decoded: Result<(Vec<HyprWorkspaceInfo>, usize), _> =
            bincode::serde::decode_from_slice(&message.payload, legacy());

        match decoded {
            Ok((data, len)) if len == message.payload.len() => Ok(data),
            Err(_) | Ok(_) => Err(HyprvisorError::BincodeError),
        }
    }
}

impl FormattedInfo for Vec<HyprWorkspaceInfo> {
    fn to_formatted_json(mut self, extra_data: &u32) -> HyprvisorResult<String> {
        self.sort_by_key(|ws| ws.id);

        let (left_half, right_half): (Vec<HyprWorkspaceInfo>, Vec<HyprWorkspaceInfo>) =
            self.iter().partition(|ws| ws.id <= *extra_data);

        self = (1..=*extra_data)
            .map(|id| {
                *left_half
                    .iter()
                    .find(|&ws| ws.id == id)
                    .unwrap_or(&HyprWorkspaceInfo::default_workspace(id))
            })
            .collect();

        self.extend(right_half);

        serde_json::to_string(&self).map_err(HyprvisorError::JsonError)
    }
}

impl TryFrom<Vec<HyprWorkspaceInfo>> for HyprvisorMessage {
    type Error = HyprvisorError;
    fn try_from(workspaces: Vec<HyprWorkspaceInfo>) -> HyprvisorResult<HyprvisorMessage> {
        let payload: Vec<u8> = encode_to_vec(workspaces, legacy())?;
        Ok(HyprvisorMessage {
            message_type: MessageType::Response,
            header: payload.len(),
            payload,
        })
    }
}

impl TryFrom<&[HyprWorkspaceInfo]> for HyprvisorMessage {
    type Error = HyprvisorError;
    fn try_from(workspaces: &[HyprWorkspaceInfo]) -> HyprvisorResult<HyprvisorMessage> {
        let payload: Vec<u8> = encode_to_vec(workspaces, legacy())?;
        Ok(HyprvisorMessage {
            message_type: MessageType::Response,
            header: payload.len(),
            payload,
        })
    }
}
