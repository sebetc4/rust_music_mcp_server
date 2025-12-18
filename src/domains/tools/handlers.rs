//! Tool handlers module.
//!
//! This module contains individual tool handler implementations.
//! Each handler is responsible for executing a specific tool's logic.
//!
//! For simple tools, the implementation can be done directly in the
//! McpServer using the #[tool] macro. This module is for more complex
//! tools that require their own dedicated handler logic.

use serde::{Deserialize, Serialize};

/// Input parameters for a generic tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    /// The name of the tool to execute.
    pub tool_name: String,

    /// The arguments to pass to the tool.
    pub arguments: serde_json::Value,
}

/// Output from a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Whether the execution was successful.
    pub success: bool,

    /// The result data from the tool.
    pub data: serde_json::Value,

    /// Optional error message if execution failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolOutput {
    /// Create a successful tool output.
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
        }
    }

    /// Create a failed tool output.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(error.into()),
        }
    }
}

// ============================================================================
// Example: Custom tool handler trait
// ============================================================================

/// Trait for implementing custom tool handlers.
///
/// Implement this trait when you need complex tool logic that doesn't fit
/// well in the simple #[tool] macro approach.
#[async_trait::async_trait]
pub trait ToolHandler: Send + Sync {
    /// Get the name of this tool.
    fn name(&self) -> &str;

    /// Get the description of this tool.
    fn description(&self) -> &str;

    /// Execute the tool with the given arguments.
    async fn execute(&self, arguments: serde_json::Value) -> ToolOutput;
}

// ============================================================================
// Example: File operations tool handler
// ============================================================================

/// A tool handler for file-related operations.
///
/// This is an example of a more complex tool handler that encapsulates
/// related functionality.
pub struct FileOperationsHandler {
    base_path: Option<String>,
}

impl FileOperationsHandler {
    /// Create a new FileOperationsHandler.
    pub fn new(base_path: Option<String>) -> Self {
        Self { base_path }
    }
}

#[async_trait::async_trait]
impl ToolHandler for FileOperationsHandler {
    fn name(&self) -> &str {
        "file_operations"
    }

    fn description(&self) -> &str {
        "Perform file operations like reading file metadata"
    }

    async fn execute(&self, arguments: serde_json::Value) -> ToolOutput {
        // Example implementation
        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match operation {
            "list" => {
                let path = self.base_path.as_deref().unwrap_or(".");

                // In a real implementation, you would list files here
                ToolOutput::success(serde_json::json!({
                    "path": path,
                    "message": "File listing would be performed here"
                }))
            }
            _ => ToolOutput::failure(format!("Unknown operation: {}", operation)),
        }
    }
}
