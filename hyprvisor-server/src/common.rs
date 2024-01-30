use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UnixStream;
use tokio::sync::Mutex;

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

pub trait HyprvisorListener {
    fn prepare_listener(&mut self);
    async fn start_listener(&mut self, subscribers: Arc<Mutex<Subscribers>>);
}
