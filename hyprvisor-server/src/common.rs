use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::UnixStream;

pub type Subscribers = HashMap<SubscriptionID, HashMap<u32, UnixStream>>;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum SubscriptionID {
    WORKSPACE,
    WINDOW,
    SINKVOLUME,
    SOURCEVOLUME,
}

#[derive(Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub pid: u32,
    pub name: String,
}

// pub struct HyprvisorData {
//     pub workspace_info: HashMap<String, bool>,
//     pub window_title: String,
//     pub sink_volume: Option<u32>,
//     pub source_volume: Option<u32>,
// }

// impl HyprvisorData {
//     pub fn new() -> Self {
//         HyprvisorData {
//             workspace_info: HashMap::new(),
//             window_title: "".to_string(),
//             sink_volume: None,
//             source_volume: None,
//         }
//     }
// }
