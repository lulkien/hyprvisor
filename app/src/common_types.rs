use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::io;
use tokio::net::UnixStream;

// This result type will be used everywhere
pub type HResult<T> = std::result::Result<T, HyprvisorError>;

#[allow(unused)]
#[derive(Debug)]
pub enum HyprvisorError {
    SerdeError,
    IoError,
    StreamError,
    DaemonRunning,
    NoDaemon,
    NoSubscribers,
    FalseAlarm,
}

impl From<io::Error> for HyprvisorError {
    fn from(_: io::Error) -> Self {
        HyprvisorError::IoError
    }
}

impl From<serde_json::Error> for HyprvisorError {
    fn from(_: serde_json::Error) -> Self {
        HyprvisorError::SerdeError
    }
}

pub type Subscriber = HashMap<SubscriptionID, HashMap<u32, UnixStream>>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum SubscriptionID {
    Workspaces,
    Window,
}

impl Display for SubscriptionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SubscriptionID::Workspaces => write!(f, "Workspaces"),
            SubscriptionID::Window => write!(f, "Window"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientInfo {
    pub process_id: u32,
    pub subscription_id: SubscriptionID,
}

impl ClientInfo {
    pub fn new(process_id: u32, subscription_id: SubscriptionID) -> Self {
        ClientInfo {
            process_id,
            subscription_id,
        }
    }
}

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
