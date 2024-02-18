use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::net::UnixStream;
use tokio::sync::Mutex;

pub type Subscribers = HashMap<SubscriptionID, HashMap<u32, UnixStream>>;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Deserialize, Serialize)]
#[allow(unused)]
pub enum SubscriptionID {
    Workspace,
    Window,
    SinkVolume,
    SourceVolume,
}

impl fmt::Display for SubscriptionID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubscriptionID::Workspace => write!(f, "Workspace"),
            SubscriptionID::Window => write!(f, "Window"),
            SubscriptionID::SinkVolume => write!(f, "SinkVolume"),
            SubscriptionID::SourceVolume => write!(f, "SourceVolume"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SubscriptionInfo {
    pub pid: u32,
    pub subscription_id: SubscriptionID,
}

pub(crate) trait HyprvisorListener {
    fn prepare_listener(&mut self);
    async fn start_listener(&mut self, subscribers: Arc<Mutex<Subscribers>>);
}

#[allow(dead_code)]
#[derive(PartialEq)]
pub(crate) enum HyprEvent {
    WorkspaceCreated,
    WorkspaceChanged,
    WorkspaceDestroyed,
    WindowChanged,
    Window2Changed,
    InvalidEvent,
    // More events will be handle in the future
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub(crate) struct WorkspaceInfo {
    pub id: u32,
    pub name: String,
    pub occupied: bool,
    pub active: bool,
}

impl WorkspaceInfo {
    pub(crate) fn new() -> Self {
        WorkspaceInfo {
            id: 0,
            name: "".to_string(),
            occupied: false,
            active: false,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub(crate) struct WindowInfo {
    pub class: String,
    pub title: String,
}

impl WindowInfo {
    pub(crate) fn new() -> Self {
        WindowInfo {
            class: "".to_string(),
            title: "".to_string(),
        }
    }
}