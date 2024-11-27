use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SubscriptionID {
    Workspaces = 0,
    Window = 1,
    Wifi = 2,
    Bluetooth = 3,
    Invalid = 255,
}

impl From<SubscriptionID> for u8 {
    fn from(value: SubscriptionID) -> Self {
        value as u8
    }
}

impl From<u8> for SubscriptionID {
    fn from(value: u8) -> Self {
        match value {
            0 => SubscriptionID::Workspaces,
            1 => SubscriptionID::Window,
            2 => SubscriptionID::Wifi,
            3 => SubscriptionID::Bluetooth,
            _ => SubscriptionID::Invalid,
        }
    }
}

impl Display for SubscriptionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SubscriptionID::Workspaces => write!(f, "Workspaces"),
            SubscriptionID::Window => write!(f, "Window"),
            SubscriptionID::Wifi => write!(f, "Wifi"),
            SubscriptionID::Bluetooth => write!(f, "Bluetooth"),
            SubscriptionID::Invalid => write!(f, "Invalid"),
        }
    }
}
