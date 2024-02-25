use serde::{Deserialize, Serialize};

pub(crate) enum HyprSocketType {
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
    InvalidEvent,
    // More events will be handle in the future
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HyprWinInfo {
    pub class: String,
    pub title: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HyprWorkspaceInfo {
    pub id: u32,
    pub occupied: bool,
    pub active: bool,
}
