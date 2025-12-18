//! Transport error types.

use thiserror::Error;

/// Result type for transport operations.
pub type TransportResult<T> = Result<T, TransportError>;

/// Errors that can occur in transport operations.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Failed to bind to address.
    #[error("Failed to bind to {address}: {source}")]
    BindError {
        address: String,
        #[source]
        source: std::io::Error,
    },

    /// Connection error.
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// IO error during transport.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Protocol error.
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Server initialization error.
    #[error("Server initialization error: {0}")]
    InitError(String),

    /// HTTP-specific error.
    #[error("HTTP error: {0}")]
    HttpError(String),

    /// Service error from rmcp.
    #[error("Service error: {0}")]
    ServiceError(String),
}

impl TransportError {
    /// Create a bind error.
    pub fn bind(address: impl Into<String>, source: std::io::Error) -> Self {
        Self::BindError {
            address: address.into(),
            source,
        }
    }

    /// Create a connection error.
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::ConnectionError(msg.into())
    }

    /// Create a protocol error.
    pub fn protocol(msg: impl Into<String>) -> Self {
        Self::ProtocolError(msg.into())
    }

    /// Create an initialization error.
    pub fn init(msg: impl Into<String>) -> Self {
        Self::InitError(msg.into())
    }

    /// Create an HTTP error.
    pub fn http(msg: impl Into<String>) -> Self {
        Self::HttpError(msg.into())
    }
}
