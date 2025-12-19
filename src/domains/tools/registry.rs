//! Tool Registry - central registration and dispatch for all tools.
//!
//! This module provides:
//! - A registry of all available tools
//! - HTTP dispatch for tool calls (when http feature is enabled)
//! - Tool metadata for listing

use std::sync::Arc;
#[cfg(feature = "http")]
use tracing::warn;

use rmcp::model::Tool;

use crate::core::config::Config;
use crate::domains::tools::definitions::MbIdentifyRecordTool;

use super::definitions::{
    FsListDirTool, FsRenameTool, MbArtistTool, MbCoverDownloadTool, MbLabelTool, MbRecordingTool,
    MbReleaseTool, MbWorkTool, ReadMetadataTool, WriteMetadataTool,
};

// ============================================================================
// Tool Registry
// ============================================================================

/// Tool registry - manages all available tools.
///
/// This struct provides a central point for:
/// - Listing all available tools
/// - Dispatching HTTP tool calls (when http feature is enabled)
pub struct ToolRegistry {
    config: Arc<Config>,
}

impl ToolRegistry {
    /// Create a new tool registry.
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// Get all tool names.
    pub fn tool_names(&self) -> Vec<&'static str> {
        vec![
            FsListDirTool::NAME,
            FsRenameTool::NAME,
            ReadMetadataTool::NAME,
            WriteMetadataTool::NAME,
            MbArtistTool::NAME,
            MbCoverDownloadTool::NAME,
            MbIdentifyRecordTool::NAME,
            MbLabelTool::NAME,
            MbRecordingTool::NAME,
            MbReleaseTool::NAME,
            MbWorkTool::NAME,
        ]
    }

    /// Get all tools as Tool models (metadata).
    ///
    /// This is the single source of truth for all available tools.
    /// Both HTTP and STDIO/TCP transports use this to get tool metadata.
    pub fn get_all_tools() -> Vec<Tool> {
        vec![
            FsListDirTool::to_tool(),
            FsRenameTool::to_tool(),
            MbArtistTool::to_tool(),
            MbCoverDownloadTool::to_tool(),
            MbIdentifyRecordTool::to_tool(),
            MbLabelTool::to_tool(),
            MbRecordingTool::to_tool(),
            MbReleaseTool::to_tool(),
            MbWorkTool::to_tool(),
            ReadMetadataTool::to_tool(),
            WriteMetadataTool::to_tool(),
        ]
    }

    /// Dispatch an HTTP tool call to the appropriate handler.
    ///
    /// This is used by the HTTP transport to call tools.
    #[cfg(feature = "http")]
    pub fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match name {
            FsListDirTool::NAME => FsListDirTool::http_handler(arguments, self.config.clone()),
            FsRenameTool::NAME => FsRenameTool::http_handler(arguments, self.config.clone()),
            MbArtistTool::NAME => MbArtistTool::http_handler(arguments),
            MbCoverDownloadTool::NAME => {
                MbCoverDownloadTool::http_handler(arguments, self.config.clone())
            }
            MbIdentifyRecordTool::NAME => {
                MbIdentifyRecordTool::http_handler(arguments, self.config.clone())
            }
            MbLabelTool::NAME => MbLabelTool::http_handler(arguments),
            MbRecordingTool::NAME => MbRecordingTool::http_handler(arguments),
            MbReleaseTool::NAME => MbReleaseTool::http_handler(arguments),
            MbWorkTool::NAME => MbWorkTool::http_handler(arguments),
            ReadMetadataTool::NAME => ReadMetadataTool::http_handler(arguments, self.config.clone()),
            WriteMetadataTool::NAME => WriteMetadataTool::http_handler(arguments, self.config.clone()),
            _ => {
                warn!("Unknown tool requested: {}", name);
                Err(format!("Unknown tool: {}", name))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Arc<Config> {
        Arc::new(Config::default())
    }

    #[test]
    fn test_registry_tool_names() {
        let registry = ToolRegistry::new(test_config());
        let names = registry.tool_names();
        assert_eq!(names.len(), 11);
        assert!(names.contains(&"fs_list_dir"));
        assert!(names.contains(&"fs_rename"));
        assert!(names.contains(&"mb_artist_search"));
        assert!(names.contains(&"mb_cover_download"));
        assert!(names.contains(&"mb_identify_record"));
        assert!(names.contains(&"mb_label_search"));
        assert!(names.contains(&"mb_recording_search"));
        assert!(names.contains(&"mb_release_search"));
        assert!(names.contains(&"mb_work_search"));
        assert!(names.contains(&"read_metadata"));
        assert!(names.contains(&"write_metadata"));
    }

    #[cfg(feature = "http")]
    #[test]
    fn test_registry_call_echo() {
        let registry = ToolRegistry::new(test_config());
        let result = registry.call_tool("fs_list_dir", serde_json::json!({ "path": "test" }));
        assert!(result.is_ok());
    }

    #[cfg(feature = "http")]
    #[test]
    fn test_registry_call_unknown() {
        let registry = ToolRegistry::new(test_config());
        let result = registry.call_tool("unknown", serde_json::json!({}));
        assert!(result.is_err());
    }
}
