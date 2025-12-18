//! Write metadata tool definition.
//!
//! A tool that writes/updates audio file metadata (ID3 tags, etc.) using lofty.

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

/// Parameters for the write metadata tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WriteMetadataParams {
    /// Path to the audio file to modify.
    pub path: String,

    /// Title of the track
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Artist name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,

    /// Album name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,

    /// Album artist
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_artist: Option<String>,

    /// Year of release
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,

    /// Track number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<u32>,

    /// Total tracks in album
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_total: Option<u32>,

    /// Genre
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,

    /// Comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// If true, clear all existing tags before writing new ones
    #[serde(default)]
    pub clear_existing: bool,
}

// ============================================================================
// Tool Definition
// ============================================================================

/// Write metadata tool - writes/updates audio file metadata using lofty.
pub struct WriteMetadataTool;

impl WriteMetadataTool {
    /// Tool name as registered in MCP.
    pub const NAME: &'static str = "write_metadata";

    /// Tool description shown to clients.
    pub const DESCRIPTION: &'static str = "Write or update metadata tags in audio files (MP3, FLAC, M4A, etc.). \
         Supports title, artist, album, year, track number, genre, and more. \
         Only provided fields will be updated.";

    /// Execute the tool logic (for STDIO/TCP transport via rmcp).
    #[instrument(skip_all, fields(path = %params.path))]
    pub fn execute(params: &WriteMetadataParams, config: &Config) -> CallToolResult {
        info!("Write metadata tool called for path: {}", params.path);

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
        let mut tagged_file = match lofty::read_from_path(&path) {
            Ok(file) => file,
            Err(e) => {
                warn!("Failed to read audio file: {}", e);
                return CallToolResult::error(vec![Content::text(format!(
                    "Failed to read audio file: {}",
                    e
                ))]);
            }
        };

        // Get or create primary tag
        let tag = if params.clear_existing {
            // Clear existing and create new tag
            tagged_file.clear();
            match tagged_file.primary_tag_mut() {
                Some(t) => t,
                None => {
                    // If no tag exists, we need to create one
                    let tag_type = tagged_file.primary_tag_type();
                    let new_tag = lofty::tag::Tag::new(tag_type);
                    tagged_file.insert_tag(new_tag);
                    tagged_file.primary_tag_mut().expect("Just inserted tag")
                }
            }
        } else {
            match tagged_file.primary_tag_mut() {
                Some(t) => t,
                None => {
                    // Create new tag if none exists
                    let tag_type = tagged_file.primary_tag_type();
                    let new_tag = lofty::tag::Tag::new(tag_type);
                    tagged_file.insert_tag(new_tag);
                    tagged_file.primary_tag_mut().expect("Just inserted tag")
                }
            }
        };

        let mut updated_fields = Vec::new();

        // Update title
        if let Some(title) = &params.title {
            tag.set_title(title.clone());
            updated_fields.push(format!("Title: {}", title));
        }

        // Update artist
        if let Some(artist) = &params.artist {
            tag.set_artist(artist.clone());
            updated_fields.push(format!("Artist: {}", artist));
        }

        // Update album
        if let Some(album) = &params.album {
            tag.set_album(album.clone());
            updated_fields.push(format!("Album: {}", album));
        }

        // Update album artist
        if let Some(album_artist) = &params.album_artist {
            tag.insert_text(lofty::tag::ItemKey::AlbumArtist, album_artist.clone());
            updated_fields.push(format!("Album Artist: {}", album_artist));
        }

        // Update year
        if let Some(year) = params.year {
            tag.set_year(year);
            updated_fields.push(format!("Year: {}", year));
        }

        // Update track number
        if let Some(track) = params.track {
            tag.set_track(track);
            updated_fields.push(format!("Track: {}", track));
        }

        // Update track total
        if let Some(track_total) = params.track_total {
            tag.set_track_total(track_total);
            updated_fields.push(format!("Track Total: {}", track_total));
        }

        // Update genre
        if let Some(genre) = &params.genre {
            tag.set_genre(genre.clone());
            updated_fields.push(format!("Genre: {}", genre));
        }

        // Update comment
        if let Some(comment) = &params.comment {
            tag.set_comment(comment.clone());
            updated_fields.push(format!("Comment: {}", comment));
        }

        // Save changes to file

        let write_options = lofty::config::WriteOptions::default();

        if let Err(e) = tagged_file.save_to_path(&path, write_options) {
            warn!("Failed to save metadata: {}", e);
            return CallToolResult::error(vec![Content::text(format!(
                "Failed to save metadata: {}",
                e
            ))]);
        }

        let mut response = format!("Successfully updated metadata for: {}\n\n", params.path);

        if params.clear_existing {
            response.push_str("⚠️  All existing tags were cleared\n\n");
        }

        response.push_str("Updated fields:\n");
        if updated_fields.is_empty() {
            response.push_str("  (no fields were updated)\n");
        } else {
            for field in &updated_fields {
                response.push_str(&format!("  • {}\n", field));
            }
        }

        info!(
            "Successfully wrote metadata to {} ({} fields updated)",
            params.path,
            if updated_fields.is_empty() {
                0
            } else {
                updated_fields.len()
            }
        );

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

        info!("Write metadata tool (HTTP) called for path: {}", path);

        let params: WriteMetadataParams = serde_json::from_value(arguments)
            .map_err(|e| format!("Failed to parse parameters: {}", e))?;

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
            input_schema: cached_schema_for_type::<WriteMetadataParams>(),
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
                let params: WriteMetadataParams =
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
    use tempfile::TempDir;

    fn test_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_write_metadata_nonexistent() {
        let params = WriteMetadataParams {
            path: "/nonexistent/audio/file.mp3".to_string(),
            title: Some("Test".to_string()),
            artist: None,
            album: None,
            album_artist: None,
            year: None,
            track: None,
            track_total: None,
            genre: None,
            comment: None,
            clear_existing: false,
        };

        let config = test_config();
        let result = WriteMetadataTool::execute(&params, &config);
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn test_write_metadata_not_a_file() {
        let temp_dir = TempDir::new().unwrap();

        let params = WriteMetadataParams {
            path: temp_dir.path().to_string_lossy().to_string(),
            title: Some("Test".to_string()),
            artist: None,
            album: None,
            album_artist: None,
            year: None,
            track: None,
            track_total: None,
            genre: None,
            comment: None,
            clear_existing: false,
        };

        let config = test_config();
        let result = WriteMetadataTool::execute(&params, &config);
        assert!(result.is_error.unwrap_or(false));
    }

    #[cfg(feature = "http")]
    #[test]
    fn test_write_metadata_http_handler_missing_path() {
        let args = serde_json::json!({
            "title": "Test Title"
        });

        let config = Arc::new(test_config());
        let result = WriteMetadataTool::http_handler(args, config);
        assert!(result.is_err());
    }
}
