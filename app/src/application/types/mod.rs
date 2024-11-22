pub mod client_info;
pub mod subscription_id;

use std::collections::HashMap;
use tokio::net::unix::OwnedWriteHalf;

pub use client_info::ClientInfo;
pub use subscription_id::SubscriptionID;

pub type Subscriber = HashMap<SubscriptionID, HashMap<u32, OwnedWriteHalf>>;
