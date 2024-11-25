pub mod listener;
pub mod types;

pub use listener::response_to_subscription;
pub use listener::start_bluetooth_listener;

use once_cell::sync::Lazy;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::Mutex;

static BLUETOOTH_POWERED: AtomicBool = AtomicBool::new(false);

static BLUETOOTH_DEVICES: Lazy<Arc<Mutex<Vec<types::BluetoothDeviceInfo>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

const POLLING_INTERVAL: u64 = 500;
const REBOOT_IWD_DELAY: u64 = 2500;
const MAX_ATTEMPT_RETRY: usize = 10;
