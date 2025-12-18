//! Prompt-specific error types.

use thiserror::Error;

/// Errors that can occur during prompt operations.
#[derive(Debug, Error)]
pub enum PromptError {
    /// The requested prompt was not found.
    #[error("Prompt not found: {0}")]
    NotFound(String),

    /// Required argument is missing.
    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    /// Invalid argument value.
    #[error("Invalid argument value for '{0}': {1}")]
    InvalidArgument(String, String),

    /// Template rendering failed.
    #[error("Template error: {0}")]
    TemplateError(String),

    /// An internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl PromptError {
    /// Create a new "not found" error.
    pub fn not_found(name: impl Into<String>) -> Self {
        Self::NotFound(name.into())
    }

    /// Create a new "missing argument" error.
    pub fn missing_argument(arg: impl Into<String>) -> Self {
        Self::MissingArgument(arg.into())
    }

    /// Create a new "invalid argument" error.
    pub fn invalid_argument(arg: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidArgument(arg.into(), reason.into())
    }

    /// Create a new "template" error.
    pub fn template(msg: impl Into<String>) -> Self {
        Self::TemplateError(msg.into())
    }

    /// Create a new "internal" error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
