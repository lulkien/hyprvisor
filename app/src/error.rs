use std::{fmt::Display, io, result::Result};

pub type HyprvisorResult<T> = Result<T, HyprvisorError>;

#[derive(Debug)]
pub enum HyprvisorError {
    DaemonRunning,
    NoDaemon,
    JsonError(serde_json::Error),
    BincodeError(bincode::Error),
    IoError(io::Error),
    IpcError,
    ParseError,
    NoSubscriber,
    WifiError,
    BluetoothError,
    FalseAlarm,
    LoggerError(fern::InitError),
    InvalidMessage,
    InvalidResponse,
    InvalidSubscription,
}

impl From<io::Error> for HyprvisorError {
    fn from(err: io::Error) -> Self {
        HyprvisorError::IoError(err)
    }
}

impl From<serde_json::Error> for HyprvisorError {
    fn from(err: serde_json::Error) -> Self {
        HyprvisorError::JsonError(err)
    }
}

impl From<bincode::Error> for HyprvisorError {
    fn from(err: bincode::Error) -> Self {
        HyprvisorError::BincodeError(err)
    }
}

impl From<fern::InitError> for HyprvisorError {
    fn from(err: fern::InitError) -> Self {
        HyprvisorError::LoggerError(err)
    }
}

impl Display for HyprvisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprvisorError::DaemonRunning => write!(f, "Daemon is already running"),
            HyprvisorError::NoDaemon => write!(f, "No daemon found"),
            HyprvisorError::JsonError(err) => write!(f, "Json error: {err}"),
            HyprvisorError::BincodeError(err) => write!(f, "Bincode error: {err}"),
            HyprvisorError::IpcError => write!(f, "Inter-processes communication error"),
            HyprvisorError::IoError(err) => write!(f, "IO error: {err}"),
            HyprvisorError::ParseError => write!(f, "Parse error"),
            HyprvisorError::NoSubscriber => write!(f, "No subscriber"),
            HyprvisorError::FalseAlarm => write!(f, "False alarm"),
            HyprvisorError::WifiError => write!(f, "Wifi error"),
            HyprvisorError::BluetoothError => write!(f, "Bluetooth error"),
            HyprvisorError::LoggerError(err) => write!(f, "Logger error: {err}"),
            HyprvisorError::InvalidMessage => write!(f, "Invalid message"),
            HyprvisorError::InvalidResponse => write!(f, "Invalid response"),
            HyprvisorError::InvalidSubscription => write!(f, "Invalid subscription"),
        }
    }
}
