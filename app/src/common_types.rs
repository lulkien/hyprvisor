use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use tokio::net::UnixStream;

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

pub type Subscriber = HashMap<SubscriptionID, HashMap<u32, UnixStream>>;
