//! Tool-specific error types.

use thiserror::Error;

/// Errors that can occur during tool operations.
#[derive(Debug, Error)]
pub enum ToolError {
    /// The requested tool was not found.
    #[error("Tool not found: {0}")]
    NotFound(String),

    /// Invalid arguments were provided to the tool.
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// The tool execution failed.
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// The tool timed out during execution.
    #[error("Tool execution timed out")]
    Timeout,

    /// An internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ToolError {
    /// Create a new "not found" error.
    pub fn not_found(name: impl Into<String>) -> Self {
        Self::NotFound(name.into())
    }

    /// Create a new "invalid arguments" error.
    pub fn invalid_arguments(msg: impl Into<String>) -> Self {
        Self::InvalidArguments(msg.into())
    }

    /// Create a new "execution failed" error.
    pub fn execution_failed(msg: impl Into<String>) -> Self {
        Self::ExecutionFailed(msg.into())
    }

    /// Create a new "internal" error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
