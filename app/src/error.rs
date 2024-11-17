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
    FalseAlarm,
    LoggerError,
    InvalidMessage,
    InvalidResponse,
}

impl From<io::Error> for HyprvisorError {
    fn from(value: io::Error) -> Self {
        HyprvisorError::IoError(value)
    }
}

impl From<serde_json::Error> for HyprvisorError {
    fn from(value: serde_json::Error) -> Self {
        HyprvisorError::JsonError(value)
    }
}

impl From<bincode::Error> for HyprvisorError {
    fn from(value: bincode::Error) -> Self {
        HyprvisorError::BincodeError(value)
    }
}

impl From<std::num::ParseIntError> for HyprvisorError {
    fn from(_: std::num::ParseIntError) -> Self {
        HyprvisorError::ParseError
    }
}

impl From<fern::InitError> for HyprvisorError {
    fn from(_: fern::InitError) -> Self {
        HyprvisorError::LoggerError
    }
}

impl Display for HyprvisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprvisorError::DaemonRunning => write!(f, "Daemon is already running"),
            HyprvisorError::NoDaemon => write!(f, "No daemon found"),
            HyprvisorError::JsonError(err) => write!(f, "Serde json error: {}", err),
            HyprvisorError::BincodeError(err) => write!(f, "Serde bincode error: {}", err),
            HyprvisorError::IpcError => write!(f, "Inter-processes communication error"),
            HyprvisorError::IoError(err) => write!(f, "IO error: {}", err),
            HyprvisorError::ParseError => write!(f, "Parse error"),
            HyprvisorError::NoSubscriber => write!(f, "No subscriber"),
            HyprvisorError::FalseAlarm => write!(f, "False alarm"),
            HyprvisorError::LoggerError => write!(f, "Cannot init logger"),
            HyprvisorError::InvalidMessage => write!(f, "Invalid message"),
            HyprvisorError::InvalidResponse => write!(f, "Invalid response"),
        }
    }
}
