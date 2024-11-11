use crate::types::Subscriber;

use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

pub static SUBSCRIBERS: Lazy<Arc<Mutex<Subscriber>>> =
    Lazy::new(|| Arc::new(Mutex::new(Subscriber::new())));

pub static BUFFER_SIZE: Lazy<usize> = Lazy::new(|| 8192);
