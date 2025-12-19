//! MusicBrainz Cover Art download tool.
//!
//! This tool downloads cover art images for music releases from the Cover Art Archive.
//! Supports multiple thumbnail sizes with intelligent fallback strategies.

use futures::FutureExt;
use musicbrainz_rs::entity::coverart::{Coverart, CoverartImage, ImageType};
use musicbrainz_rs::entity::release::Release;
use musicbrainz_rs::entity::CoverartResponse;
use musicbrainz_rs::FetchCoverart;
use rmcp::{
    ErrorData as McpError,
    handler::server::tool::{ToolCallContext, ToolRoute, schema_for_type},
    model::{CallToolResult, Content, Tool},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::core::config::Config;
use crate::core::security::validate_path;

use super::common::{error_result, is_mbid, structured_result};

// ============================================================================
// Tool Parameters
// ============================================================================

/// Parameters for cover art download operations.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MbCoverDownloadParams {
    /// MusicBrainz Release ID (UUID format).
    #[schemars(description = "MusicBrainz Release ID (MBID) in UUID format")]
    pub mbid: String,

    /// Directory path where cover will be saved.
    #[schemars(description = "Target directory path (must be within allowed root)")]
    pub path: String,

    /// Filename without extension (default: "cover").
    #[serde(default = "default_filename")]
    #[schemars(description = "Output filename without extension (default: 'cover')")]
    pub filename: String,

    /// Preferred thumbnail size: "250", "500", "1200", or "original".
    #[serde(default = "default_thumbnail_size")]
    #[schemars(description = "Thumbnail size: 250, 500, 1200, or original (default: 500)")]
    pub thumbnail_size: String,

    /// Whether to overwrite existing file.
    #[serde(default)]
    #[schemars(description = "Overwrite existing file if present (default: false)")]
    pub overwrite: bool,
}

fn default_filename() -> String {
    "cover".to_string()
}

fn default_thumbnail_size() -> String {
    "500".to_string()
}

// ============================================================================
// Structured Output
// ============================================================================

/// Structured output for cover download results.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoverDownloadResult {
    pub success: bool,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub image_type: String,
    pub thumbnail_size: String,
    pub source_url: String,
}

// ============================================================================
// Tool Implementation
// ============================================================================

/// MusicBrainz Cover Art Download Tool implementation.
#[derive(Debug, Clone)]
pub struct MbCoverDownloadTool;

impl MbCoverDownloadTool {
    /// Tool name as registered in MCP.
    pub const NAME: &'static str = "mb_cover_download";

    /// Tool description shown to clients.
    pub const DESCRIPTION: &'static str = "Download cover art images for music releases from the Cover Art Archive. \
         Supports multiple thumbnail sizes (250, 500, 1200, or original) with intelligent fallback. \
         Prioritizes Front cover but falls back to other available images. \
         Returns structured data with file path, size, and image metadata.";

    pub fn new() -> Self {
        Self
    }

    /// Execute the tool logic (for STDIO/TCP transport via rmcp).
    pub fn execute(params: &MbCoverDownloadParams, config: &Config) -> CallToolResult {
        info!(
            "Cover download tool called for MBID: {}, path: {}",
            params.mbid, params.path
        );

        // 1. Validate MBID format
        if !is_mbid(&params.mbid) {
            warn!("Invalid MBID format: {}", params.mbid);
            return error_result("Invalid MBID format (expected UUID)");
        }

        // 2. Validate path with security
        let dir_path = match validate_path(&params.path, config) {
            Ok(p) => p,
            Err(e) => {
                warn!("Path security validation failed: {}", e);
                return CallToolResult::error(vec![Content::text(format!(
                    "Path security validation failed: {}",
                    e
                ))]);
            }
        };

        // 3. Verify it's a directory
        if !dir_path.is_dir() {
            warn!("Path is not a directory: {}", params.path);
            return error_result(&format!("Path is not a directory: {}", params.path));
        }

        // 4. Validate thumbnail_size
        if !matches!(
            params.thumbnail_size.as_str(),
            "250" | "500" | "1200" | "original"
        ) {
            warn!("Invalid thumbnail size: {}", params.thumbnail_size);
            return error_result("Invalid thumbnail size (use 250, 500, 1200, or original)");
        }

        // 5. Fetch coverart metadata from MusicBrainz Cover Art Archive
        info!("Fetching cover art metadata for MBID: {}", params.mbid);
        let coverart = match Release::fetch_coverart().id(&params.mbid).execute() {
            Ok(CoverartResponse::Json(coverart)) => coverart,
            Ok(CoverartResponse::Url(_)) => {
                error!("Unexpected URL response (expected JSON metadata)");
                return error_result("Unexpected URL response (expected JSON metadata)");
            }
            Err(e) => {
                error!("Failed to fetch cover art: {:?}", e);
                return error_result(&format!("Failed to fetch cover art: {}", e));
            }
        };

        // 6. Select the best image (Front prioritized)
        let selected_image = match Self::select_best_image(&coverart) {
            Ok(img) => img,
            Err(e) => {
                warn!("No suitable image found: {}", e);
                return error_result(&format!("No suitable image found: {}", e));
            }
        };

        // 7. Get URL for requested size with fallback
        let (image_url, actual_size) =
            Self::get_image_url(selected_image, &params.thumbnail_size);

        // Validate URL
        if image_url.is_empty() {
            error!("Empty image URL received from API");
            return error_result("Invalid image URL received from Cover Art Archive");
        }

        info!(
            "Selected image URL ({}): {}",
            actual_size,
            image_url.chars().take(60).collect::<String>()
        );

        // 8. Download the image with proper HTTP client configuration
        let client = match reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(10))
            .timeout(std::time::Duration::from_secs(30))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to build HTTP client: {:?}", e);
                return error_result(&format!("Failed to build HTTP client: {}", e));
            }
        };

        // Convert HTTP URLs to HTTPS for Cover Art Archive
        let secure_url = if image_url.starts_with("http://coverartarchive.org") {
            image_url.replace("http://", "https://")
        } else {
            image_url.clone()
        };

        info!("Downloading from: {}", secure_url);

        let image_bytes = match client.get(&secure_url).send() {
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    error!("HTTP request failed with status: {} for URL: {}", status, secure_url);
                    return error_result(&format!(
                        "Failed to download image: HTTP {} - URL: {}",
                        status, secure_url
                    ));
                }
                match response.bytes() {
                    Ok(bytes) => {
                        if bytes.is_empty() {
                            error!("Received empty response from: {}", secure_url);
                            return error_result("Failed to download image: Empty response");
                        }
                        bytes
                    }
                    Err(e) => {
                        error!("Failed to read response bytes: {:?}", e);
                        return error_result(&format!("Failed to read image data: {}", e));
                    }
                }
            }
            Err(e) => {
                error!("HTTP request failed for URL {}: {:?}", secure_url, e);
                return error_result(&format!(
                    "Failed to download image from {}: {}",
                    secure_url, e
                ));
            }
        };

        // 9. Determine file extension from URL
        let extension = Self::detect_extension(&image_url);
        let full_filename = format!("{}.{}", params.filename, extension);
        let file_path = dir_path.join(&full_filename);

        // 10. Check if file exists
        if file_path.exists() && !params.overwrite {
            warn!("File already exists: {}", file_path.display());
            return error_result(&format!(
                "File already exists: {}. Use overwrite=true to replace",
                file_path.display()
            ));
        }

        // 11. Write the file
        if let Err(e) = std::fs::write(&file_path, &image_bytes) {
            error!("Failed to write file: {:?}", e);
            return error_result(&format!("Failed to write file: {}", e));
        }

        // 12. Build result
        let image_type = if selected_image.front {
            "Front".to_string()
        } else if selected_image.back {
            "Back".to_string()
        } else {
            selected_image
                .types
                .first()
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|| "Unknown".to_string())
        };

        let result = CoverDownloadResult {
            success: true,
            file_path: file_path.display().to_string(),
            file_size_bytes: image_bytes.len() as u64,
            image_type: image_type.clone(),
            thumbnail_size: actual_size.clone(),
            source_url: secure_url,
        };

        let summary = format!(
            "Downloaded {} cover ({}) to {} ({} bytes)",
            image_type, actual_size, file_path.display(), result.file_size_bytes
        );

        info!("{}", summary);

        structured_result(summary, result)
    }

    /// HTTP handler for this tool (for HTTP transport).
    #[cfg(feature = "http")]
    pub fn http_handler(
        arguments: serde_json::Value,
        config: Arc<Config>,
    ) -> Result<serde_json::Value, String> {
        let mbid = arguments
            .get("mbid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid 'mbid' parameter".to_string())?
            .to_string();

        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid 'path' parameter".to_string())?
            .to_string();

        let filename = arguments
            .get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("cover")
            .to_string();

        let thumbnail_size = arguments
            .get("thumbnail_size")
            .and_then(|v| v.as_str())
            .unwrap_or("500")
            .to_string();

        let overwrite = arguments
            .get("overwrite")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let params = MbCoverDownloadParams {
            mbid,
            path,
            filename,
            thumbnail_size,
            overwrite,
        };

        // Use std::thread::spawn to avoid nested runtime panic.
        // musicbrainz_rs and reqwest::blocking both create their own runtime.
        let handle = std::thread::spawn(move || Self::execute(&params, &config));

        let result = handle
            .join()
            .map_err(|_| "Thread panicked during cover download".to_string())?;

        let mut response = serde_json::json!({
            "content": result.content,
            "isError": result.is_error.unwrap_or(false)
        });

        // Include structured_content if present
        if let Some(structured) = result.structured_content {
            response.as_object_mut().unwrap().insert(
                "structuredContent".to_string(),
                structured,
            );
        }

        Ok(response)
    }

    /// Create a Tool model for this tool (metadata).
    pub fn to_tool() -> Tool {
        Tool {
            name: Self::NAME.into(),
            description: Some(Self::DESCRIPTION.into()),
            input_schema: schema_for_type::<MbCoverDownloadParams>(),
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
                let params: MbCoverDownloadParams =
                    serde_json::from_value(serde_json::Value::Object(args))
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

                // Use std::thread::spawn to avoid nested runtime panic.
                // musicbrainz_rs and reqwest::blocking both create their own runtime.
                let handle = std::thread::spawn(move || Self::execute(&params, &config));

                let result = handle.join().map_err(|_| {
                    McpError::internal_error("Thread panicked".to_string(), None)
                })?;

                Ok(result)
            }
            .boxed()
        })
    }

    // ========================================================================
    // Helper Functions
    // ========================================================================

    /// Select the best image (Front prioritized, fallback to first available).
    fn select_best_image(coverart: &Coverart) -> Result<&CoverartImage, &'static str> {
        if coverart.images.is_empty() {
            return Err("No images available");
        }

        // Priority 1: Image marked as "front"
        if let Some(img) = coverart.images.iter().find(|img| img.front) {
            return Ok(img);
        }

        // Priority 2: Image with type "Front"
        if let Some(img) = coverart.images.iter().find(|img| {
            img.types
                .iter()
                .any(|t| matches!(t, ImageType::Front))
        }) {
            return Ok(img);
        }

        // Fallback: First available image
        coverart
            .images
            .first()
            .ok_or("No images available after fallback")
    }

    /// Get URL for requested size with intelligent fallback.
    /// Supports both new format (res_250, res_500, res_1200) and legacy format (small, large).
    fn get_image_url(image: &CoverartImage, requested_size: &str) -> (String, String) {
        match requested_size {
            "250" => {
                // Try 250 (new format), fallback to small (legacy), then 500, 1200, original
                image
                    .thumbnails
                    .res_250
                    .clone()
                    .or_else(|| image.thumbnails.small.clone())
                    .map(|url| (url, "250".to_string()))
                    .or_else(|| {
                        image
                            .thumbnails
                            .res_500
                            .clone()
                            .or_else(|| image.thumbnails.large.clone())
                            .map(|url| (url, "500".to_string()))
                    })
                    .or_else(|| {
                        image
                            .thumbnails
                            .res_1200
                            .clone()
                            .map(|url| (url, "1200".to_string()))
                    })
                    .unwrap_or_else(|| (image.image.clone(), "original".to_string()))
            }
            "500" => {
                // Try 500 (new format), fallback to large (legacy), then 1200, 250/small, original
                image
                    .thumbnails
                    .res_500
                    .clone()
                    .or_else(|| image.thumbnails.large.clone())
                    .map(|url| (url, "500".to_string()))
                    .or_else(|| {
                        image
                            .thumbnails
                            .res_1200
                            .clone()
                            .map(|url| (url, "1200".to_string()))
                    })
                    .or_else(|| {
                        image
                            .thumbnails
                            .res_250
                            .clone()
                            .or_else(|| image.thumbnails.small.clone())
                            .map(|url| (url, "250".to_string()))
                    })
                    .unwrap_or_else(|| (image.image.clone(), "original".to_string()))
            }
            "1200" => {
                // Try 1200, fallback to large/500 (legacy), then original
                image
                    .thumbnails
                    .res_1200
                    .clone()
                    .map(|url| (url, "1200".to_string()))
                    .or_else(|| {
                        image
                            .thumbnails
                            .res_500
                            .clone()
                            .or_else(|| image.thumbnails.large.clone())
                            .map(|url| (url, "500".to_string()))
                    })
                    .unwrap_or_else(|| (image.image.clone(), "original".to_string()))
            }
            "original" | _ => (image.image.clone(), "original".to_string()),
        }
    }

    /// Detect file extension from URL.
    fn detect_extension(url: &str) -> String {
        if url.ends_with(".png") {
            "png"
        } else if url.ends_with(".jpg") || url.ends_with(".jpeg") {
            "jpg"
        } else if url.ends_with(".gif") {
            "gif"
        } else if url.ends_with(".webp") {
            "webp"
        } else {
            // Fallback: jpg (most common format)
            "jpg"
        }
        .to_string()
    }
}

impl Default for MbCoverDownloadTool {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_defaults() {
        let json = r#"{"mbid": "65c70b9f-fdef-4bc0-a5b6-ac4e34252d3c", "path": "/tmp"}"#;
        let params: MbCoverDownloadParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.filename, "cover");
        assert_eq!(params.thumbnail_size, "500");
        assert_eq!(params.overwrite, false);
    }

    #[test]
    fn test_params_custom() {
        let json = r#"{
            "mbid": "65c70b9f-fdef-4bc0-a5b6-ac4e34252d3c",
            "path": "/tmp",
            "filename": "album_art",
            "thumbnail_size": "1200",
            "overwrite": true
        }"#;
        let params: MbCoverDownloadParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.filename, "album_art");
        assert_eq!(params.thumbnail_size, "1200");
        assert_eq!(params.overwrite, true);
    }

    #[test]
    fn test_mbid_validation() {
        // Valid MBIDs
        assert!(is_mbid("65c70b9f-fdef-4bc0-a5b6-ac4e34252d3c"));
        assert!(is_mbid("76df3287-6cda-33eb-8e9a-044b5e15ffdd"));

        // Invalid MBIDs
        assert!(!is_mbid("not-a-uuid"));
        assert!(!is_mbid("65c70b9f-fdef-4bc0-a5b6")); // Too short
        assert!(!is_mbid("65c70b9f-fdef-4bc0-a5b6-ac4e34252d3cXX")); // Too long
        assert!(!is_mbid("65c70b9f-fdef-4bc0-a5b6-ac4e34252d3")); // Missing char
        assert!(!is_mbid("65c70b9f_fdef_4bc0_a5b6_ac4e34252d3c")); // Wrong separator
    }

    #[test]
    fn test_extension_detection() {
        assert_eq!(
            MbCoverDownloadTool::detect_extension("https://example.com/image.jpg"),
            "jpg"
        );
        assert_eq!(
            MbCoverDownloadTool::detect_extension("https://example.com/image.jpeg"),
            "jpg"
        );
        assert_eq!(
            MbCoverDownloadTool::detect_extension("https://example.com/image.png"),
            "png"
        );
        assert_eq!(
            MbCoverDownloadTool::detect_extension("https://example.com/image.gif"),
            "gif"
        );
        assert_eq!(
            MbCoverDownloadTool::detect_extension("https://example.com/image.webp"),
            "webp"
        );
        assert_eq!(
            MbCoverDownloadTool::detect_extension("https://example.com/noext"),
            "jpg"
        ); // Fallback
    }

    #[test]
    fn test_get_image_url_legacy_format() {
        use musicbrainz_rs::entity::coverart::{CoverartImage, Thumbnail};

        // Create a CoverartImage with legacy format (small/large only)
        let thumbnail = Thumbnail {
            small: Some("http://example.com/image-250.jpg".to_string()),
            large: Some("http://example.com/image-500.jpg".to_string()),
            res_250: None,
            res_500: None,
            res_1200: None,
        };

        let image = CoverartImage {
            approved: true,
            back: false,
            comment: "Test".to_string(),
            edit: 123,
            front: true,
            id: 456,
            image: "http://example.com/image-original.jpg".to_string(),
            thumbnails: thumbnail,
            types: vec![],
        };

        // Test 250px request - should use "small" field
        let (url, size) = MbCoverDownloadTool::get_image_url(&image, "250");
        assert_eq!(url, "http://example.com/image-250.jpg");
        assert_eq!(size, "250");

        // Test 500px request - should use "large" field
        let (url, size) = MbCoverDownloadTool::get_image_url(&image, "500");
        assert_eq!(url, "http://example.com/image-500.jpg");
        assert_eq!(size, "500");

        // Test 1200px request - should fallback to "large" (500)
        let (url, size) = MbCoverDownloadTool::get_image_url(&image, "1200");
        assert_eq!(url, "http://example.com/image-500.jpg");
        assert_eq!(size, "500");

        // Test original request
        let (url, size) = MbCoverDownloadTool::get_image_url(&image, "original");
        assert_eq!(url, "http://example.com/image-original.jpg");
        assert_eq!(size, "original");
    }

    #[test]
    fn test_get_image_url_new_format() {
        use musicbrainz_rs::entity::coverart::{CoverartImage, Thumbnail};

        // Create a CoverartImage with new format (res_250, res_500, res_1200)
        let thumbnail = Thumbnail {
            small: None,
            large: None,
            res_250: Some("http://example.com/image-250.jpg".to_string()),
            res_500: Some("http://example.com/image-500.jpg".to_string()),
            res_1200: Some("http://example.com/image-1200.jpg".to_string()),
        };

        let image = CoverartImage {
            approved: true,
            back: false,
            comment: "Test".to_string(),
            edit: 123,
            front: true,
            id: 456,
            image: "http://example.com/image-original.jpg".to_string(),
            thumbnails: thumbnail,
            types: vec![],
        };

        // Test all sizes
        let (url, size) = MbCoverDownloadTool::get_image_url(&image, "250");
        assert_eq!(url, "http://example.com/image-250.jpg");
        assert_eq!(size, "250");

        let (url, size) = MbCoverDownloadTool::get_image_url(&image, "500");
        assert_eq!(url, "http://example.com/image-500.jpg");
        assert_eq!(size, "500");

        let (url, size) = MbCoverDownloadTool::get_image_url(&image, "1200");
        assert_eq!(url, "http://example.com/image-1200.jpg");
        assert_eq!(size, "1200");
    }

    #[test]
    fn test_get_image_url_mixed_format() {
        use musicbrainz_rs::entity::coverart::{CoverartImage, Thumbnail};

        // Create a CoverartImage with mixed format (some legacy, some new)
        let thumbnail = Thumbnail {
            small: Some("http://example.com/image-250-legacy.jpg".to_string()),
            large: None,
            res_250: None,
            res_500: Some("http://example.com/image-500-new.jpg".to_string()),
            res_1200: None,
        };

        let image = CoverartImage {
            approved: true,
            back: false,
            comment: "Test".to_string(),
            edit: 123,
            front: true,
            id: 456,
            image: "http://example.com/image-original.jpg".to_string(),
            thumbnails: thumbnail,
            types: vec![],
        };

        // Test 250px - should prefer res_250 (None) then fallback to small (legacy)
        let (url, size) = MbCoverDownloadTool::get_image_url(&image, "250");
        assert_eq!(url, "http://example.com/image-250-legacy.jpg");
        assert_eq!(size, "250");

        // Test 500px - should use res_500 (new format available)
        let (url, size) = MbCoverDownloadTool::get_image_url(&image, "500");
        assert_eq!(url, "http://example.com/image-500-new.jpg");
        assert_eq!(size, "500");
    }

    // Network tests (require actual internet connection, run with --ignored)
    #[ignore]
    #[test]
    fn test_download_real_cover() {
        use tempfile::TempDir;

        // Respect rate limiting
        std::thread::sleep(std::time::Duration::from_millis(1500));

        let temp_dir = TempDir::new().unwrap();
        let params = MbCoverDownloadParams {
            mbid: "65c70b9f-fdef-4bc0-a5b6-ac4e34252d3c".to_string(),
            path: temp_dir.path().to_string_lossy().to_string(),
            filename: "test_cover".to_string(),
            thumbnail_size: "250".to_string(),
            overwrite: false,
        };

        let config = Config::default();
        let result = MbCoverDownloadTool::execute(&params, &config);

        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );

        // Verify file was created
        let files: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert!(
            !files.is_empty(),
            "Expected at least one file to be created"
        );

        // Verify structured content
        if let Some(structured) = result.structured_content {
            let cover_result: CoverDownloadResult =
                serde_json::from_value(structured).unwrap();
            assert!(cover_result.success);
            assert!(cover_result.file_size_bytes > 0);
            assert!(cover_result.file_path.contains("test_cover"));
        }
    }

    #[ignore]
    #[test]
    fn test_download_original_size() {
        use tempfile::TempDir;

        std::thread::sleep(std::time::Duration::from_millis(1500));

        let temp_dir = TempDir::new().unwrap();
        let params = MbCoverDownloadParams {
            mbid: "65c70b9f-fdef-4bc0-a5b6-ac4e34252d3c".to_string(),
            path: temp_dir.path().to_string_lossy().to_string(),
            filename: "original_cover".to_string(),
            thumbnail_size: "original".to_string(),
            overwrite: false,
        };

        let config = Config::default();
        let result = MbCoverDownloadTool::execute(&params, &config);

        assert!(!result.is_error.unwrap_or(true), "Expected success");
    }
}
