pub mod listener;
pub mod types;

pub use listener::start_bluetooth_listener;

use once_cell::sync::Lazy;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::Mutex;
use types::BluetoothDeviceInfo;

static CONNECTED_DEVICES: Lazy<Arc<Mutex<HashSet<BluetoothDeviceInfo>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));

const POLLING_INTERVAL: u64 = 500;
const REBOOT_IWD_DELAY: u64 = 2500;
const MAX_ATTEMPT_RETRY: usize = 10;
