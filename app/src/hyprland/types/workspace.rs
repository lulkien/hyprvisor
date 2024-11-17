use super::FormattedInfo;
use crate::{
    error::{HyprvisorError, HyprvisorResult},
    ipc::message::HyprvisorMessage,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct HyprWorkspaceInfo {
    pub id: u32,
    pub occupied: bool,
    pub active: bool,
}

impl TryFrom<HyprvisorMessage> for Vec<HyprWorkspaceInfo> {
    type Error = HyprvisorError;
    fn try_from(message: HyprvisorMessage) -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
        if !message.is_valid() {
            return Err(HyprvisorError::InvalidMessage);
        }
        bincode::deserialize(&message.payload).map_err(HyprvisorError::BincodeError)
    }
}

impl FormattedInfo for Vec<HyprWorkspaceInfo> {
    fn to_formatted_json(self, extra_data: &u32) -> HyprvisorResult<String> {
        if self.len() > *extra_data as usize {
            return serde_json::to_string(&self).map_err(HyprvisorError::JsonError);
        }

        let mut table: HashMap<u32, HyprWorkspaceInfo> =
            self.into_iter().map(|ws| (ws.id, ws)).collect();

        (1..=*extra_data).for_each(|id| {
            table.entry(id).or_insert_with(|| HyprWorkspaceInfo {
                id,
                occupied: false,
                active: false,
            });
        });

        let mut modified: Vec<HyprWorkspaceInfo> = table.into_values().collect();
        modified.sort_by_key(|info| info.id);

        serde_json::to_string(&modified).map_err(HyprvisorError::JsonError)
    }
}
