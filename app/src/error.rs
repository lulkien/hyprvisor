use std::{io, result::Result};

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
    RegexError(regex::Error),
    ParseError,
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

impl From<regex::Error> for HyprvisorError {
    fn from(value: regex::Error) -> Self {
        HyprvisorError::RegexError(value)
    }
}

impl From<std::num::ParseIntError> for HyprvisorError {
    fn from(_: std::num::ParseIntError) -> Self {
        HyprvisorError::ParseError
    }
}
