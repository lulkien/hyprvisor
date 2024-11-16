use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    error::{HyprvisorError, HyprvisorResult},
    ipc::message::HyprvisorMessage,
};

pub(super) enum HyprSocketType {
    Event,
    Command,
}

#[derive(Debug, PartialEq)]
pub(super) enum HyprEvent {
    WorkspaceCreated,
    WorkspaceChanged,
    WorkspaceDestroyed,
    WindowChanged,
    Window2Changed,
    IgnoredEvent,
    // More events will be handle in the future
}

pub(super) struct HyprEventList(Vec<HyprEvent>);

impl From<&[u8]> for HyprEventList {
    fn from(buffer: &[u8]) -> Self {
        let mut evt_list: Vec<HyprEvent> = String::from_utf8_lossy(buffer)
            .lines()
            .map(|line| match line.split(">>").next().unwrap_or_default() {
                "activewindow" => HyprEvent::WindowChanged,
                "workspace" => HyprEvent::WorkspaceChanged,
                "activewindowv2" => HyprEvent::Window2Changed,
                "createworkspace" => HyprEvent::WorkspaceCreated,
                "destroyworkspace" => HyprEvent::WorkspaceDestroyed,
                _ => HyprEvent::IgnoredEvent,
            })
            .collect();
        evt_list.dedup();
        Self(evt_list)
    }
}

impl HyprEventList {
    pub fn contains(&self, event: &HyprEvent) -> bool {
        self.0.contains(event)
    }

    pub fn contains_at_least(&self, events: &[&HyprEvent]) -> bool {
        events.iter().any(|&event| self.contains(event))
    }
}

#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct HyprWindowInfo {
    pub class: String,
    pub title: String,
}

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct HyprWorkspaceInfo {
    pub id: u32,
    pub occupied: bool,
    pub active: bool,
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

impl TryFrom<HyprvisorMessage> for Vec<HyprWorkspaceInfo> {
    type Error = HyprvisorError;
    fn try_from(message: HyprvisorMessage) -> HyprvisorResult<Vec<HyprWorkspaceInfo>> {
        if !message.is_valid() {
            return Err(HyprvisorError::InvalidMessage);
        }
        bincode::deserialize(&message.payload).map_err(HyprvisorError::BincodeError)
    }
}

pub trait FormattedInfo {
    fn to_formatted_json(self, extra_data: &u32) -> HyprvisorResult<String>;
}

impl FormattedInfo for HyprWindowInfo {
    fn to_formatted_json(mut self, extra_data: &u32) -> HyprvisorResult<String> {
        if let Some(title) = self.title.get(..*extra_data as usize) {
            self.title = format!("{}...", String::from_utf8_lossy(title.as_bytes()));
        }

        serde_json::to_string(&self).map_err(HyprvisorError::JsonError)
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
