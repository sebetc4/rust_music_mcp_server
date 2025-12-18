//! Resource-specific error types.

use thiserror::Error;

/// Errors that can occur during resource operations.
#[derive(Debug, Error)]
pub enum ResourceError {
    /// The requested resource was not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Access to the resource was denied.
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// The resource URI is invalid.
    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    /// An I/O error occurred while accessing the resource.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// An internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ResourceError {
    /// Create a new "not found" error.
    pub fn not_found(uri: impl Into<String>) -> Self {
        Self::NotFound(uri.into())
    }

    /// Create a new "access denied" error.
    pub fn access_denied(msg: impl Into<String>) -> Self {
        Self::AccessDenied(msg.into())
    }

    /// Create a new "invalid URI" error.
    pub fn invalid_uri(uri: impl Into<String>) -> Self {
        Self::InvalidUri(uri.into())
    }

    /// Create a new "internal" error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
