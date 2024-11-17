use std::fmt::{Display, Formatter, Result};

use crate::error::HyprvisorError;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
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
