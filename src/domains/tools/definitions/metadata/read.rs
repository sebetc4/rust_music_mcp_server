//! Read metadata tool definition.
//!
//! A tool that reads audio file metadata (ID3 tags, etc.) using lofty.

use rmcp::{
    ErrorData as McpError,
    handler::server::tool::{ToolCallContext, ToolRoute, cached_schema_for_type},
    model::{CallToolResult, Content, Tool},
};

use futures::FutureExt;
use lofty::prelude::*;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{info, instrument, warn};

use crate::core::config::Config;
use crate::core::security::validate_path;

// ============================================================================
// Tool Parameters
// ============================================================================

/// Parameters for the read metadata tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReadMetadataParams {
    /// Path to the audio file to read.
    pub path: String,

    /// Include technical audio properties (bitrate, sample rate, duration)
    #[serde(default)]
    pub include_properties: bool,
}

// ============================================================================
// Tool Definition
// ============================================================================

/// Read metadata tool - reads audio file metadata using lofty.
pub struct ReadMetadataTool;

impl ReadMetadataTool {
    /// Tool name as registered in MCP.
    pub const NAME: &'static str = "read_metadata";

    /// Tool description shown to clients.
    pub const DESCRIPTION: &'static str = "Read metadata from audio files (MP3, FLAC, M4A, etc.). Returns tags like artist, album, title, year, and optionally technical properties.";

    /// Execute the tool logic (for STDIO/TCP transport via rmcp).
    #[instrument(skip_all, fields(path = %params.path))]
    pub fn execute(params: &ReadMetadataParams, config: &Config) -> CallToolResult {
        info!("Read metadata tool called for path: {}", params.path);

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

        // Validate it's a file
        if !path.is_file() {
            warn!("Path is not a file: {}", params.path);
            return CallToolResult::error(vec![Content::text(format!(
                "Path is not a file: {}",
                params.path
            ))]);
        }

        // Read the audio file
        let tagged_file = match lofty::read_from_path(path) {
            Ok(file) => file,
            Err(e) => {
                warn!("Failed to read audio file: {}", e);
                return CallToolResult::error(vec![Content::text(format!(
                    "Failed to read audio file: {}",
                    e
                ))]);
            }
        };

        let mut response = format!("File: {}\n", params.path);
        response.push_str(&format!("Format: {:?}\n\n", tagged_file.file_type()));

        // Read primary tag
        if let Some(tag) = tagged_file.primary_tag() {
            response.push_str("=== Metadata ===\n");

            if let Some(title) = tag.title() {
                response.push_str(&format!("Title:       {}\n", title));
            }

            if let Some(artist) = tag.artist() {
                response.push_str(&format!("Artist:      {}\n", artist));
            }

            if let Some(album) = tag.album() {
                response.push_str(&format!("Album:       {}\n", album));
            }

            if let Some(album_artist) = tag.get_string(&lofty::tag::ItemKey::AlbumArtist) {
                response.push_str(&format!("AlbumArtist: {}\n", album_artist));
            }

            if let Some(year) = tag.year() {
                response.push_str(&format!("Year:        {}\n", year));
            }

            if let Some(track) = tag.track() {
                response.push_str(&format!("Track:       {}\n", track));
            }

            if let Some(genre) = tag.genre() {
                response.push_str(&format!("Genre:       {}\n", genre));
            }

            if let Some(comment) = tag.comment() {
                response.push_str(&format!("Comment:     {}\n", comment));
            }

            // Count total tags
            let tag_count = tag.item_count();
            response.push_str(&format!("\nTotal tags: {}\n", tag_count));
        } else {
            response.push_str("No metadata tags found.\n");
        }

        // Include technical properties if requested
        if params.include_properties {
            response.push_str("\n=== Audio Properties ===\n");

            let properties = tagged_file.properties();

            if let Some(duration) = properties.duration().as_secs().checked_sub(0) {
                let minutes = duration / 60;
                let seconds = duration % 60;
                response.push_str(&format!("Duration:    {}:{:02}\n", minutes, seconds));
            }

            if let Some(bitrate) = properties.audio_bitrate() {
                response.push_str(&format!("Bitrate:     {} kbps\n", bitrate));
            }

            if let Some(sample_rate) = properties.sample_rate() {
                response.push_str(&format!("Sample Rate: {} Hz\n", sample_rate));
            }

            if let Some(channels) = properties.channels() {
                let channel_str = match channels {
                    1 => "Mono",
                    2 => "Stereo",
                    _ => "Multi-channel",
                };
                response.push_str(&format!("Channels:    {} ({})\n", channels, channel_str));
            }

            if let Some(bit_depth) = properties.bit_depth() {
                response.push_str(&format!("Bit Depth:   {} bits\n", bit_depth));
            }
        }

        info!("Successfully read metadata from {}", params.path);

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

        let include_properties = arguments
            .get("include_properties")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        info!("Read metadata tool (HTTP) called for path: {}", path);

        let params = ReadMetadataParams {
            path,
            include_properties,
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
            input_schema: cached_schema_for_type::<ReadMetadataParams>(),
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
                let params: ReadMetadataParams =
                    serde_json::from_value(serde_json::Value::Object(args))
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                Ok(Self::execute(&params, &config))
            }
            .boxed()
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_read_metadata_nonexistent() {
        let params = ReadMetadataParams {
            path: "/nonexistent/audio/file.mp3".to_string(),
            include_properties: false,
        };

        let config = test_config();
        let result = ReadMetadataTool::execute(&params, &config);
        assert!(result.is_error.unwrap_or(false));
    }

    #[cfg(feature = "http")]
    #[test]
    fn test_read_metadata_http_handler_missing_param() {
        let args = serde_json::json!({
            "include_properties": true
        });

        let config = Arc::new(test_config());
        let result = ReadMetadataTool::http_handler(args, config);
        assert!(result.is_err());
    }
}
