//! Central error type for kbdanim.
//!
//! Every fallible operation in the crate returns `crate::error::Result<T>`.
//! Errors are always constructed with a human-readable message so the CLI
//! can print something actionable instead of a bare `Debug` dump.

use std::fmt;

#[derive(Debug)]
pub enum KbdAnimError {
    DeviceNotFound(String),
    PermissionDenied(String),
    InvalidColor(String),
    InvalidMode(String),
    InvalidFps(String),
    InvalidSpeed(String),
    InvalidBrightness(String),
    ConfigParse(String),
    SocketError(String),
    DeviceWrite(String),
    Io(std::io::Error),
    Other(String),
}

impl fmt::Display for KbdAnimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KbdAnimError::DeviceNotFound(s) => write!(f, "device not found: {s}"),
            KbdAnimError::PermissionDenied(s) => write!(f, "permission denied: {s}"),
            KbdAnimError::InvalidColor(s) => write!(f, "invalid color: {s}"),
            KbdAnimError::InvalidMode(s) => write!(f, "invalid animation mode: {s}"),
            KbdAnimError::InvalidFps(s) => write!(f, "invalid fps: {s}"),
            KbdAnimError::InvalidSpeed(s) => write!(f, "invalid speed: {s}"),
            KbdAnimError::InvalidBrightness(s) => write!(f, "invalid brightness: {s}"),
            KbdAnimError::ConfigParse(s) => write!(f, "config parse error: {s}"),
            KbdAnimError::SocketError(s) => write!(f, "ipc socket error: {s}"),
            KbdAnimError::DeviceWrite(s) => write!(f, "device write failed: {s}"),
            KbdAnimError::Io(e) => write!(f, "io error: {e}"),
            KbdAnimError::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for KbdAnimError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            KbdAnimError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for KbdAnimError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => KbdAnimError::DeviceNotFound(e.to_string()),
            std::io::ErrorKind::PermissionDenied => {
                KbdAnimError::PermissionDenied(e.to_string())
            }
            _ => KbdAnimError::Io(e),
        }
    }
}

pub type Result<T> = std::result::Result<T, KbdAnimError>;
