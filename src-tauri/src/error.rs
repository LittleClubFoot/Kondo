use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application result type
pub type Result<T> = std::result::Result<T, AppError>;

/// Unified application error type with proper serialization for Tauri IPC
#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Operation failed: {0}")]
    Operation(String),
}

// Implement conversions from common error types
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

// Helper function for creating errors with context
impl AppError {
    pub fn config_error(msg: impl Into<String>) -> Self {
        AppError::Config(msg.into())
    }

    pub fn validation_error(msg: impl Into<String>) -> Self {
        AppError::Validation(msg.into())
    }

    pub fn path_error(msg: impl Into<String>) -> Self {
        AppError::Path(msg.into())
    }

    pub fn operation_error(msg: impl Into<String>) -> Self {
        AppError::Operation(msg.into())
    }
}
