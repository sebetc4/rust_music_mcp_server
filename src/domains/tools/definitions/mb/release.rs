//! MusicBrainz Release search tool.
//!
//! This tool provides functionality to search for releases (albums),
//! get tracks/recordings in a release, and find all versions of a release group.

use futures::FutureExt;
use futures::future::BoxFuture;
use musicbrainz_rs::{
    Fetch, Search,
    entity::release::{Release, ReleaseSearchQuery},
    entity::release_group::{ReleaseGroup, ReleaseGroupSearchQuery},
};
use rmcp::{
    ErrorData as McpError,
    handler::server::tool::{ToolCallContext, ToolRoute, cached_schema_for_type},
    model::{CallToolResult, Tool},
};
use schemars::JsonSchema;
use serde::Deserialize;
use tracing::{debug, error, info};

use super::common::{
    default_limit, error_result, extract_year, format_date, format_duration, get_artist_name,
    is_mbid, success_result, validate_limit,
};

/// Parameters for release search operations.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MbReleaseParams {
    /// The type of search to perform.
    /// - "release": Search for releases by title
    /// - "release_recordings": Get all tracks/recordings in a release
    /// - "release_group_releases": Get all versions of a release group
    #[schemars(
        description = "Search type: 'release', 'release_recordings', or 'release_group_releases'"
    )]
    pub search_type: String,

    /// The search query string or MusicBrainz ID.
    #[schemars(description = "Search query (release/release-group title or MBID)")]
    pub query: String,

    /// Maximum number of results to return (default: 10, max: 100).
    #[schemars(description = "Maximum number of results (default: 10, max: 100)")]
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// MusicBrainz Release Search Tool implementation.
#[derive(Debug, Clone)]
pub struct MbReleaseTool;

impl MbReleaseTool {
    /// Tool name as registered in MCP.
    pub const NAME: &'static str = "mb_release_search";

    /// Tool description shown to clients.
    pub const DESCRIPTION: &'static str = "Search for releases (albums) in MusicBrainz, get track listings, and find all versions of a release group.";

    pub fn new() -> Self {
        Self
    }

    /// Execute the tool logic (for STDIO/TCP transport via rmcp).
    pub fn execute(params: &MbReleaseParams) -> CallToolResult {
        let search_type = params.search_type.clone();
        let query = params.query.clone();
        let limit = validate_limit(params.limit);

        match search_type.as_str() {
            "release" => Self::search_releases(&query, limit),
            "release_recordings" => Self::search_release_recordings(&query, limit),
            "release_group_releases" => Self::search_release_group_releases(&query, limit),
            _ => error_result(&format!(
                "Unknown search type: {}. Use 'release', 'release_recordings', or 'release_group_releases'",
                search_type
            )),
        }
    }

    /// HTTP handler for this tool (for HTTP transport).
    #[cfg(feature = "http")]
    pub fn http_handler(arguments: serde_json::Value) -> Result<serde_json::Value, String> {
        let search_type = arguments
            .get("search_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid 'search_type' parameter".to_string())?
            .to_string();

        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid 'query' parameter".to_string())?
            .to_string();

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let params = MbReleaseParams {
            search_type,
            query,
            limit,
        };

        // Use std::thread::spawn to avoid nested runtime panic.
        // musicbrainz_rs uses reqwest::blocking which creates its own runtime.
        let handle = std::thread::spawn(move || Self::execute(&params));

        let result = handle
            .join()
            .map_err(|_| "Thread panicked during release search".to_string())?;

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
            input_schema: cached_schema_for_type::<MbReleaseParams>(),
            annotations: None,
            output_schema: None,
            icons: None,
            meta: None,
            title: None,
        }
    }

    /// Create a ToolRoute for STDIO/TCP transport.
    pub fn create_route<S>() -> ToolRoute<S>
    where
        S: Send + Sync + 'static,
    {
        ToolRoute::new_dyn(Self::to_tool(), |ctx: ToolCallContext<'_, S>| {
            let args = ctx.arguments.clone().unwrap_or_default();
            async move {
                let params: MbReleaseParams =
                    serde_json::from_value(serde_json::Value::Object(args))
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

                // Use std::thread::spawn to avoid nested runtime panic.
                // musicbrainz_rs uses reqwest::blocking which creates its own runtime,
                // so we need a completely separate OS thread.
                let handle = std::thread::spawn(move || Self::execute(&params));

                let result = handle
                    .join()
                    .map_err(|_| McpError::internal_error("Thread panicked".to_string(), None))?;

                Ok(result)
            }
            .boxed()
        })
    }

    /// Main handler for HTTP transport.
    #[deprecated(note = "Use http_handler() instead")]
    pub fn handle_http(params: MbReleaseParams) -> BoxFuture<'static, CallToolResult> {
        Box::pin(async move {
            let search_type = params.search_type.clone();
            let query = params.query.clone();
            let limit = validate_limit(params.limit);

            let result = std::thread::spawn(move || {
                match search_type.as_str() {
                    "release" => Self::search_releases(&query, limit),
                    "release_recordings" => Self::search_release_recordings(&query, limit),
                    "release_group_releases" => Self::search_release_group_releases(&query, limit),
                    _ => error_result(&format!(
                        "Unknown search type: {}. Use 'release', 'release_recordings', or 'release_group_releases'",
                        search_type
                    )),
                }
            })
            .join()
            .unwrap_or_else(|e| error_result(&format!("Thread panicked: {:?}", e)));

            result
        })
    }

    /// Main handler for STDIO/TCP transport.
    pub fn handle_stdio(params: MbReleaseParams) -> BoxFuture<'static, CallToolResult> {
        Box::pin(async move {
            let search_type = params.search_type.clone();
            let query = params.query.clone();
            let limit = validate_limit(params.limit);

            let result = tokio::task::spawn_blocking(move || {
                match search_type.as_str() {
                    "release" => Self::search_releases(&query, limit),
                    "release_recordings" => Self::search_release_recordings(&query, limit),
                    "release_group_releases" => Self::search_release_group_releases(&query, limit),
                    _ => error_result(&format!(
                        "Unknown search type: {}. Use 'release', 'release_recordings', or 'release_group_releases'",
                        search_type
                    )),
                }
            })
            .await
            .unwrap_or_else(|e| error_result(&format!("Task failed: {:?}", e)));

            result
        })
    }

    /// Search for releases by title.
    pub fn search_releases(query: &str, limit: usize) -> CallToolResult {
        info!("Searching for releases matching: {}", query);

        let search_query = ReleaseSearchQuery::query_builder().release(query).build();

        let search_result = Release::search(search_query).execute();

        match search_result {
            Ok(result) => {
                let releases: Vec<_> = result.entities.into_iter().take(limit).collect();
                if releases.is_empty() {
                    return error_result(&format!("No releases found for query: {}", query));
                }

                let mut output = format!("Found {} releases:\n\n", releases.len());
                for (i, release) in releases.iter().enumerate() {
                    let artist = get_artist_name(&release.artist_credit);
                    let year = release
                        .date
                        .as_ref()
                        .and_then(|d| extract_year(&d.0))
                        .unwrap_or_else(|| "Unknown".to_string());

                    output.push_str(&format!(
                        "{}. **{}** by {} ({})\n   MBID: {}\n",
                        i + 1,
                        release.title,
                        artist,
                        year,
                        release.id,
                    ));
                    if let Some(country) = &release.country {
                        output.push_str(&format!("   Country: {}\n", country));
                    }
                    if let Some(barcode) = &release.barcode {
                        if !barcode.is_empty() {
                            output.push_str(&format!("   Barcode: {}\n", barcode));
                        }
                    }
                    output.push('\n');
                }

                success_result(output)
            }
            Err(e) => {
                error!("Release search failed: {:?}", e);
                error_result(&format!("Release search failed: {}", e))
            }
        }
    }

    /// Get all tracks/recordings in a release.
    pub fn search_release_recordings(query: &str, limit: usize) -> CallToolResult {
        info!("Getting recordings for release: {}", query);

        // Get the release MBID
        let release_id = if is_mbid(query) {
            query.to_string()
        } else {
            // Search for release first
            let search_query = ReleaseSearchQuery::query_builder().release(query).build();
            match Release::search(search_query).execute() {
                Ok(result) => {
                    if let Some(release) = result.entities.first() {
                        debug!("Found release: {} ({})", release.title, release.id);
                        release.id.clone()
                    } else {
                        return error_result(&format!("No release found matching: {}", query));
                    }
                }
                Err(e) => {
                    error!("Release lookup failed: {:?}", e);
                    return error_result(&format!("Release lookup failed: {}", e));
                }
            }
        };

        // Fetch release with recordings (media->tracks)
        match Release::fetch().id(&release_id).with_recordings().execute() {
            Ok(release) => {
                let artist = get_artist_name(&release.artist_credit);
                let mut output = format!(
                    "**{}** by {}\nMBID: {}\n\n",
                    release.title, artist, release.id
                );

                if let Some(media) = &release.media {
                    let mut track_num = 0;
                    for (disc_idx, medium) in media.iter().enumerate() {
                        if media.len() > 1 {
                            let disc_title = medium
                                .title
                                .as_ref()
                                .map(|t| format!(" - {}", t))
                                .unwrap_or_default();
                            output.push_str(&format!(
                                "\n**Disc {}{}**\n",
                                disc_idx + 1,
                                disc_title
                            ));
                        }

                        if let Some(tracks) = &medium.tracks {
                            for track in tracks.iter().take(limit) {
                                track_num += 1;
                                if let Some(ref recording) = track.recording {
                                    let duration = recording
                                        .length
                                        .map(|l| format_duration(l as u64))
                                        .unwrap_or_else(|| "--:--".to_string());

                                    let track_artist = get_artist_name(&recording.artist_credit);
                                    let artist_suffix = if track_artist != artist
                                        && track_artist != "Unknown Artist"
                                    {
                                        format!(" ({})", track_artist)
                                    } else {
                                        String::new()
                                    };

                                    output.push_str(&format!(
                                        "  {}. {} [{}]{}\n     MBID: {}\n",
                                        track_num,
                                        recording.title,
                                        duration,
                                        artist_suffix,
                                        recording.id,
                                    ));
                                }
                            }
                        }
                    }
                } else {
                    output.push_str("No track information available.\n");
                }

                success_result(output)
            }
            Err(e) => {
                error!("Failed to fetch release recordings: {:?}", e);
                error_result(&format!("Failed to fetch release recordings: {}", e))
            }
        }
    }

    /// Get all releases/versions of a release group.
    pub fn search_release_group_releases(query: &str, limit: usize) -> CallToolResult {
        info!("Getting all versions of release group: {}", query);

        // Get the release group MBID
        let release_group_id = if is_mbid(query) {
            query.to_string()
        } else {
            // Search for release group first
            let search_query = ReleaseGroupSearchQuery::query_builder()
                .release_group(query)
                .build();
            match ReleaseGroup::search(search_query).execute() {
                Ok(result) => {
                    if let Some(rg) = result.entities.first() {
                        debug!("Found release group: {} ({})", rg.title, rg.id);
                        rg.id.clone()
                    } else {
                        return error_result(&format!(
                            "No release group found matching: {}",
                            query
                        ));
                    }
                }
                Err(e) => {
                    error!("Release group lookup failed: {:?}", e);
                    return error_result(&format!("Release group lookup failed: {}", e));
                }
            }
        };

        // Fetch release group with releases
        match ReleaseGroup::fetch()
            .id(&release_group_id)
            .with_releases()
            .execute()
        {
            Ok(release_group) => {
                let artist = get_artist_name(&release_group.artist_credit);
                let mut output = format!(
                    "**{}** by {}\nRelease Group MBID: {}\n\n",
                    release_group.title, artist, release_group.id,
                );

                if let Some(releases) = &release_group.releases {
                    output.push_str(&format!(
                        "Found {} versions:\n\n",
                        releases.len().min(limit)
                    ));
                    for (i, release) in releases.iter().take(limit).enumerate() {
                        let date = release
                            .date
                            .as_ref()
                            .map(|d| format_date(&d.0))
                            .unwrap_or_else(|| "Unknown".to_string());

                        output.push_str(&format!(
                            "{}. **{}** ({})\n   MBID: {}\n",
                            i + 1,
                            release.title,
                            date,
                            release.id,
                        ));
                        if let Some(country) = &release.country {
                            output.push_str(&format!("   Country: {}\n", country));
                        }
                        output.push('\n');
                    }
                } else {
                    output.push_str("No releases found for this release group.\n");
                }

                success_result(output)
            }
            Err(e) => {
                error!("Failed to fetch release group: {:?}", e);
                error_result(&format!("Failed to fetch release group: {}", e))
            }
        }
    }
}

impl Default for MbReleaseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::RawContent;

    #[test]
    fn test_release_params_default_limit() {
        let json = r#"{"search_type": "release", "query": "Nevermind"}"#;
        let params: MbReleaseParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 10);
    }

    // Integration tests (require network, run with: cargo test -- --ignored)
    #[ignore]
    #[test]
    fn test_search_releases() {
        let result = MbReleaseTool::search_releases("Nevermind", 5);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
        let content = &result.content[0];
        if let RawContent::Text(text) = &content.raw {
            assert!(
                text.text.contains("Nevermind"),
                "Expected 'Nevermind' in result"
            );
        }
    }

    #[ignore]
    #[test]
    fn test_search_release_recordings() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        // OK Computer release MBID
        let result =
            MbReleaseTool::search_release_recordings("0d52c146-6e39-30d2-918e-cd9c7b3cbe07", 20);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
        let content = &result.content[0];
        if let RawContent::Text(text) = &content.raw {
            assert!(
                text.text.contains("OK Computer") || text.text.contains("MBID"),
                "Expected release info in result"
            );
        }
    }

    #[ignore]
    #[test]
    fn test_search_release_group_releases() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        // OK Computer release group MBID
        let result = MbReleaseTool::search_release_group_releases(
            "18079f7b-78c3-3980-b16e-c5db63cc10a5",
            10,
        );
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
    }
}
