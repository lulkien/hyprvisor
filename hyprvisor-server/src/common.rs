use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UnixStream;
use tokio::sync::Mutex;

pub type Subscribers = HashMap<SubscriptionID, HashMap<u32, UnixStream>>;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum SubscriptionID {
    Workspace,
    Window,
    SinkVolume,
    SourceVolume,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SubscriptionInfo {
    pub pid: u32,
    pub name: String,
}

pub(crate) trait HyprvisorListener {
    fn prepare_listener(&mut self);
    async fn start_listener(&mut self, subscribers: Arc<Mutex<Subscribers>>);
}

#[allow(dead_code)]
pub(crate) enum HyprEvent {
    WorkspaceCreated(String),
    WorkspaceChanged(String),
    WorkspaceDestroyed(String),
    WindowChanged((String, String)),
    Window2Changed(String),
    // More events will be handle in the future
}

#[derive(Debug, Serialize)]
pub(crate) struct WorkspaceInfo {
    pub id: u32,
    pub name: String,
    pub monitor: String,
    pub active: bool,
}

impl WorkspaceInfo {
    pub(crate) fn new() -> Self {
        WorkspaceInfo {
            id: 0,
            name: "".to_string(),
            monitor: "".to_string(),
            active: false,
        }
    }
}

#[derive(Debug, Serialize)]
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
