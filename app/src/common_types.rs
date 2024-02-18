use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::net::UnixStream;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum SubscriptionID {
    Workspaces,
    Window,
}

pub type Subscriber = HashMap<SubscriptionID, HashSet<UnixStream>>;
