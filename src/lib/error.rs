// Common error types for PPM

use std::fmt;

#[derive(Debug)]
pub enum PpmError {
    IoError(std::io::Error),
    ConfigError(String),
    NetworkError(String),
    ValidationError(String),
    SymlinkError(String),
}

impl fmt::Display for PpmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PpmError::IoError(err) => write!(f, "IO error: {}", err),
            PpmError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            PpmError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            PpmError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            PpmError::SymlinkError(msg) => write!(f, "Symlink error: {}", msg),
        }
    }
}

impl From<std::io::Error> for PpmError {
    fn from(err: std::io::Error) -> Self {
        PpmError::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, PpmError>;
