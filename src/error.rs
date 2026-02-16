//! Error types and handling for JsonQ
//!
//! Provides structured error types that can be converted to PHP exceptions

use ext_php_rs::exception::PhpException;
use std::fmt;

/// Result type for JsonQ operations
pub type Result<T> = std::result::Result<T, JsonQError>;

/// Error types for JsonQ
#[derive(Debug)]
pub enum JsonQError {
    /// IO error (files, directories)
    Io(String),
    /// Serialization/Deserialization error
    Serde(String),
    /// Security violation (path traversal, size limit, etc)
    Security(String),
    /// Path not found in JSON structure
    PathNotFound(String),
    /// Invalid operation in current state (e.g. transaction already active)
    InvalidOperation(String),
    /// General error with message
    General(String),
}

impl fmt::Display for JsonQError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO Error: {}", e),
            Self::Serde(e) => write!(f, "JSON Error: {}", e),
            Self::Security(e) => write!(f, "Security Error: {}", e),
            Self::PathNotFound(p) => write!(f, "Path not found: {}", p),
            Self::InvalidOperation(e) => write!(f, "Invalid Operation: {}", e),
            Self::General(e) => write!(f, "Error: {}", e),
        }
    }
}

impl std::error::Error for JsonQError {}

impl From<std::io::Error> for JsonQError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<serde_json::Error> for JsonQError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e.to_string())
    }
}

impl From<String> for JsonQError {
    fn from(e: String) -> Self {
        Self::General(e)
    }
}

impl JsonQError {
    /// Convert error to PHP Exception and throw it
    pub fn throw(&self) -> PhpException {
        PhpException::new(
            format!("JsonQ Error: {}", self),
            0,
            ext_php_rs::zend::ce::exception()
        )
    }
}
