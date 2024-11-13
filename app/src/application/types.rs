use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use tokio::net::UnixStream;

use crate::error::HyprvisorError;

pub type Subscriber = HashMap<SubscriptionID, HashMap<u32, UnixStream>>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SubscriptionID {
    Workspaces = 0,
    Window = 1,
    Wireless = 2,
}

impl From<SubscriptionID> for u8 {
    fn from(value: SubscriptionID) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for SubscriptionID {
    type Error = HyprvisorError;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(SubscriptionID::Workspaces),
            1 => Ok(SubscriptionID::Window),
            2 => Ok(SubscriptionID::Wireless),
            _ => Err(HyprvisorError::ParseError),
        }
    }
}

impl Display for SubscriptionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SubscriptionID::Workspaces => write!(f, "Workspaces"),
            SubscriptionID::Window => write!(f, "Window"),
            SubscriptionID::Wireless => write!(f, "Wireless"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientInfo {
    pub subscription_id: SubscriptionID,
    pub process_id: u32,
}

impl From<ClientInfo> for Vec<u8> {
    fn from(client_info: ClientInfo) -> Self {
        let mut buffer = Vec::new();

        buffer.push(u8::from(client_info.subscription_id));
        buffer.extend_from_slice(&client_info.process_id.to_le_bytes());

        buffer
    }
}

impl TryFrom<&[u8]> for ClientInfo {
    type Error = HyprvisorError;
    fn try_from(buffer: &[u8]) -> std::result::Result<Self, Self::Error> {
        if buffer.len() < (size_of::<SubscriptionID>() + size_of::<u32>()) {
            return Err(HyprvisorError::ParseError);
        }

        let subscription_id = SubscriptionID::try_from(buffer[0])?;
        let process_id = u32::from_le_bytes(buffer[1..(1 + size_of::<u32>())].try_into().unwrap());

        Ok(ClientInfo {
            subscription_id,
            process_id,
        })
    }
}

impl ClientInfo {
    pub fn new(process_id: u32, subscription_id: SubscriptionID) -> Self {
        ClientInfo {
            subscription_id,
            process_id,
        }
    }

    pub fn byte_size() -> usize {
        size_of::<u8>() + size_of::<u32>()
    }
}
