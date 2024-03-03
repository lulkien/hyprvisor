use std::{fmt::Display, io, result::Result};

// This result type will be used everywhere
pub type HyprvisorResult<T> = Result<T, HyprvisorError>;

#[allow(unused)]
#[derive(Debug)]
pub enum HyprvisorError {
    DaemonRunning,
    NoDaemon,
    SerdeError(serde_json::Error),
    IoError(io::Error),
    StreamError,
    ParseError,
    NoSubscriber,
    FalseAlarm,
}

impl From<io::Error> for HyprvisorError {
    fn from(value: io::Error) -> Self {
        HyprvisorError::IoError(value)
    }
}

impl From<serde_json::Error> for HyprvisorError {
    fn from(value: serde_json::Error) -> Self {
        HyprvisorError::SerdeError(value)
    }
}

impl From<std::num::ParseIntError> for HyprvisorError {
    fn from(_: std::num::ParseIntError) -> Self {
        HyprvisorError::ParseError
    }
}

impl Display for HyprvisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprvisorError::DaemonRunning => write!(f, "Daemon is already running"),
            HyprvisorError::NoDaemon => write!(f, "No daemon found"),
            HyprvisorError::SerdeError(err) => write!(f, "Serde error: {}", err),
            HyprvisorError::IoError(err) => write!(f, "IO error: {}", err),
            HyprvisorError::StreamError => write!(f, "Stream error"),
            HyprvisorError::ParseError => write!(f, "Parse error"),
            HyprvisorError::NoSubscriber => write!(f, "No subscriber"),
            HyprvisorError::FalseAlarm => write!(f, "False alarm"),
        }
    }
}
