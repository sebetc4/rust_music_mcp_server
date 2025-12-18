//! List directory tool definition.
//!
//! A tool that lists files and directories in a given path.

use futures::FutureExt;
use rmcp::{
    ErrorData as McpError,
    handler::server::tool::{ToolCallContext, ToolRoute, cached_schema_for_type},
    model::{CallToolResult, Content, Tool},
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use tracing::{info, instrument, warn};

use crate::core::config::Config;
use crate::core::security::validate_path;

// ============================================================================
// Tool Parameters
// ============================================================================

/// Parameters for the list directory tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FSListDirParams {
    /// Path to the directory to list.
    pub path: String,

    /// Include hidden files (starting with '.')
    #[serde(default)]
    pub include_hidden: bool,

    /// Show additional details (size, type, permissions)
    #[serde(default)]
    pub detailed: bool,
}

// ============================================================================
// Tool Definition
// ============================================================================

/// List directory tool - lists files and directories in a given path.
pub struct FsListDirTool;

impl FsListDirTool {
    /// Tool name as registered in MCP.
    pub const NAME: &'static str = "fs_list_dir";

    /// Tool description shown to clients.
    pub const DESCRIPTION: &'static str = "List files and directories in a given path. Returns names, types, and optionally sizes and permissions.";

    /// Execute the tool logic (for STDIO/TCP transport via rmcp).
    #[instrument(skip_all, fields(path = %params.path))]
    pub fn execute(params: &FSListDirParams, config: &Config) -> CallToolResult {
        info!("List directory tool called for path: {}", params.path);

        // Validate path security first
        let path = match validate_path(&params.path, config) {
            Ok(p) => p,
            Err(e) => {
                warn!("Path security validation failed: {}", e);
                return CallToolResult::error(vec![Content::text(format!(
                    "Path security validation failed: {}",
                    e
                ))]);
            }
        };

        // Validate it's a directory
        if !path.is_dir() {
            warn!("Path is not a directory: {}", params.path);
            return CallToolResult::error(vec![Content::text(format!(
                "Path is not a directory: {}",
                params.path
            ))]);
        }

        // Read directory entries
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read directory: {}", e);
                return CallToolResult::error(vec![Content::text(format!(
                    "Failed to read directory: {}",
                    e
                ))]);
            }
        };

        let mut result_lines = Vec::new();
        let mut file_count = 0;
        let mut dir_count = 0;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Error reading entry: {}", e);
                    continue;
                }
            };

            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            // Skip hidden files if not requested
            if !params.include_hidden && name.starts_with('.') {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to get metadata for {}: {}", name, e);
                    continue;
                }
            };

            if params.detailed {
                let entry_type = if metadata.is_dir() {
                    dir_count += 1;
                    "DIR "
                } else if metadata.is_symlink() {
                    "LINK"
                } else {
                    file_count += 1;
                    "FILE"
                };

                let size = if metadata.is_file() {
                    format_size(metadata.len())
                } else {
                    "-".to_string()
                };

                result_lines.push(format!("{:4}  {:>10}  {}", entry_type, size, name));
            } else {
                if metadata.is_dir() {
                    dir_count += 1;
                    result_lines.push(format!("{}/", name));
                } else {
                    file_count += 1;
                    result_lines.push(name.to_string());
                }
            }
        }

        // Sort entries
        result_lines.sort();

        // Build response
        let mut response = format!("Directory: {}\n", params.path);
        if params.detailed {
            response.push_str("\nType  Size        Name\n");
            response.push_str("----  ----------  ----\n");
        }
        response.push_str(&result_lines.join("\n"));
        response.push_str(&format!(
            "\n\nTotal: {} directories, {} files",
            dir_count, file_count
        ));

        info!("Listed {} entries in {}", result_lines.len(), params.path);

        CallToolResult::success(vec![Content::text(response)])
    }

    /// HTTP handler for this tool (for HTTP transport).
    #[cfg(feature = "http")]
    pub fn http_handler(
        arguments: serde_json::Value,
        config: Arc<Config>,
    ) -> Result<serde_json::Value, String> {
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid 'path' parameter".to_string())?
            .to_string();

        let include_hidden = arguments
            .get("include_hidden")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let detailed = arguments
            .get("detailed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        info!("List directory tool (HTTP) called for path: {}", path);

        let params = FSListDirParams {
            path,
            include_hidden,
            detailed,
        };

        let result = Self::execute(&params, &config);

        Ok(serde_json::json!({
            "content": result.content,
            "isError": result.is_error.unwrap_or(false)
        }))
    }

    /// Create a Tool model for this tool (metadata).
    pub fn to_tool() -> Tool {
        Tool {
            name: Self::NAME.into(),
            description: Some(Self::DESCRIPTION.into()),
            input_schema: cached_schema_for_type::<FSListDirParams>(),
            annotations: None,
            output_schema: None,
            icons: None,
            meta: None,
            title: None,
        }
    }

    /// Create a ToolRoute for STDIO/TCP transport.
    pub fn create_route<S>(config: Arc<Config>) -> ToolRoute<S>
    where
        S: Send + Sync + 'static,
    {
        ToolRoute::new_dyn(Self::to_tool(), move |ctx: ToolCallContext<'_, S>| {
            let args = ctx.arguments.clone().unwrap_or_default();
            let config = config.clone();
            async move {
                let params: FSListDirParams =
                    serde_json::from_value(serde_json::Value::Object(args))
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                Ok(Self::execute(&params, &config))
            }
            .boxed()
        })
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format file size in human-readable format.
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn test_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_list_dir_execute() {
        // Create a temporary directory with test files
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(temp_path.join("file1.txt"), "content").unwrap();
        fs::write(temp_path.join("file2.txt"), "content").unwrap();
        fs::create_dir(temp_path.join("subdir")).unwrap();

        let params = FSListDirParams {
            path: temp_path.to_string_lossy().to_string(),
            include_hidden: false,
            detailed: false,
        };

        let config = test_config();
        let result = FsListDirTool::execute(&params, &config);
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        // Extract text from result
        let text = match &result.content[0].raw {
            rmcp::model::RawContent::Text(text) => &text.text,
            _ => panic!("Expected text content"),
        };

        assert!(text.contains("file1.txt"));
        assert!(text.contains("file2.txt"));
        assert!(text.contains("subdir/"));
    }

    #[test]
    fn test_list_dir_nonexistent() {
        let params = FSListDirParams {
            path: "/nonexistent/path/12345".to_string(),
            include_hidden: false,
            detailed: false,
        };

        let config = test_config();
        let result = FsListDirTool::execute(&params, &config);
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn test_list_dir_detailed() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        fs::write(temp_path.join("test.txt"), "hello world").unwrap();

        let params = FSListDirParams {
            path: temp_path.to_string_lossy().to_string(),
            include_hidden: false,
            detailed: true,
        };

        let config = test_config();
        let result = FsListDirTool::execute(&params, &config);
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        // Extract text from result
        let text = match &result.content[0].raw {
            rmcp::model::RawContent::Text(text) => &text.text,
            _ => panic!("Expected text content"),
        };

        assert!(text.contains("FILE"));
        assert!(text.contains("test.txt"));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[cfg(feature = "http")]
    #[test]
    fn test_list_dir_http_handler() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        fs::write(temp_path.join("test.txt"), "content").unwrap();

        let args = serde_json::json!({
            "path": temp_path.to_string_lossy(),
            "include_hidden": false,
            "detailed": false
        });

        let config = Arc::new(test_config());
        let result = FsListDirTool::http_handler(args, config);
        assert!(result.is_ok());
    }

    #[cfg(feature = "http")]
    #[test]
    fn test_list_dir_http_handler_missing_param() {
        let args = serde_json::json!({
            "include_hidden": true
        });

        let config = Arc::new(test_config());
        let result = FsListDirTool::http_handler(args, config);
        assert!(result.is_err());
    }
}
