// error.rs
//! Error types for the Panasonic AC library

use std::io;
use thiserror::Error;

/// Library-specific error types
#[derive(Debug, Error)]
pub enum Error {
    /// Invalid state error with description
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Invalid value error with description
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    /// Invalid checksum error
    #[error("Invalid checksum")]
    InvalidChecksum,

    /// Invalid data error with description
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Unsupported feature for specific model
    #[error("Feature '{feature}' not supported{}", .model.as_ref().map_or(String::new(), |m| format!(" by model '{}'", m)))]
    UnsupportedFeature {
        feature: String,
        model: Option<String>,
    },

    /// Error with I/O operations
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Hardware error
    #[error("Hardware error: {0}")]
    HardwareError(String),

    /// GPIO error
    #[error("GPIO error: {0}")]
    GpioError(String),
}

/// Specialized Result type for the library
pub type Result<T> = std::result::Result<T, Error>;
