use std::{io, result::Result};

// This result type will be used everywhere
pub type HyprvisorResult<T> = Result<T, HyprvisorError>;

#[allow(unused)]
#[derive(Debug, PartialEq)]
pub enum HyprvisorError {
    SerdeError,
    IoError,
    StreamError,
    DaemonRunning,
    NoDaemon,
}

impl From<io::Error> for HyprvisorError {
    fn from(_: io::Error) -> Self {
        HyprvisorError::IoError
    }
}

impl From<serde_json::Error> for HyprvisorError {
    fn from(_: serde_json::Error) -> Self {
        HyprvisorError::SerdeError
    }
}
