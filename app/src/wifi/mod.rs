pub mod iwd_listener;
pub mod types;

pub use iwd_listener::response_to_subscription;
pub use iwd_listener::start_wifi_listener;

use crate::wifi::types::WifiInfo;

use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

const POLLING_INTERVAL: u64 = 500;

static CURRENT_WIFI: Lazy<Arc<Mutex<WifiInfo>>> =
    Lazy::new(|| Arc::new(Mutex::new(WifiInfo::default())));
