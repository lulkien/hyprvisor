pub mod listener;
pub mod types;
pub mod utils;
pub mod window;
pub mod workspaces;

pub use listener::start_hyprland_listener;

use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::{HyprWindowInfo, HyprWorkspaceInfo};

static CURRENT_WINDOW: Lazy<Arc<Mutex<HyprWindowInfo>>> =
    Lazy::new(|| Arc::new(Mutex::new(HyprWindowInfo::default())));

static CURRENT_WORKSPACES: Lazy<Arc<Mutex<Vec<HyprWorkspaceInfo>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));
