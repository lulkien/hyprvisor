use serde::{Deserialize, Serialize};

pub enum HyprSocketType {
    Event,
    Command,
}

#[derive(Debug, PartialEq)]
pub enum HyprEvent {
    WorkspaceCreated,
    WorkspaceChanged,
    WorkspaceDestroyed,
    WindowChanged,
    Window2Changed,
    InvalidEvent,
    // More events will be handle in the future
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct HyprWinInfo {
    pub class: String,
    pub title: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct HyprWorkspaceInfo {
    pub id: u32,
    pub occupied: bool,
    pub active: bool,
}
