//! MusicBrainz Recording search tool.
//!
//! This tool provides functionality to search for recordings (tracks/songs)
//! and find which releases contain a specific recording.

use futures::FutureExt;
use futures::future::BoxFuture;
use musicbrainz_rs::{
    Fetch, Search,
    entity::recording::{Recording, RecordingSearchQuery},
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
    default_limit, error_result, format_date, format_duration, get_artist_name, is_mbid,
    success_result, validate_limit,
};

/// Parameters for recording search operations.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MbRecordingParams {
    /// The type of search to perform.
    /// - "recording": Search for recordings by title
    /// - "recording_releases": Find all releases containing a specific recording
    #[schemars(description = "Search type: 'recording' or 'recording_releases'")]
    pub search_type: String,

    /// The search query string or MusicBrainz ID.
    #[schemars(description = "Search query (recording title or MBID)")]
    pub query: String,

    /// Maximum number of results to return (default: 10, max: 100).
    #[schemars(description = "Maximum number of results (default: 10, max: 100)")]
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// MusicBrainz Recording Search Tool implementation.
#[derive(Debug, Clone)]
pub struct MbRecordingTool;

impl MbRecordingTool {
    /// Tool name as registered in MCP.
    pub const NAME: &'static str = "mb_recording_search";

    /// Tool description shown to clients.
    pub const DESCRIPTION: &'static str =
        "Search for recordings (tracks/songs) in MusicBrainz and find which releases contain them.";

    pub fn new() -> Self {
        Self
    }

    /// Execute the tool logic (for STDIO/TCP transport via rmcp).
    pub fn execute(params: &MbRecordingParams) -> CallToolResult {
        let search_type = params.search_type.clone();
        let query = params.query.clone();
        let limit = validate_limit(params.limit);

        match search_type.as_str() {
            "recording" => Self::search_recordings(&query, limit),
            "recording_releases" => Self::search_recording_releases(&query, limit),
            _ => error_result(&format!(
                "Unknown search type: {}. Use 'recording' or 'recording_releases'",
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

        let params = MbRecordingParams {
            search_type,
            query,
            limit,
        };

        // Use std::thread::spawn to avoid nested runtime panic.
        // musicbrainz_rs uses reqwest::blocking which creates its own runtime.
        let handle = std::thread::spawn(move || Self::execute(&params));

        let result = handle
            .join()
            .map_err(|_| "Thread panicked during recording search".to_string())?;

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
            input_schema: cached_schema_for_type::<MbRecordingParams>(),
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
                let params: MbRecordingParams =
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
    pub fn handle_http(params: MbRecordingParams) -> BoxFuture<'static, CallToolResult> {
        Box::pin(async move {
            let search_type = params.search_type.clone();
            let query = params.query.clone();
            let limit = validate_limit(params.limit);

            let result = std::thread::spawn(move || match search_type.as_str() {
                "recording" => Self::search_recordings(&query, limit),
                "recording_releases" => Self::search_recording_releases(&query, limit),
                _ => error_result(&format!(
                    "Unknown search type: {}. Use 'recording' or 'recording_releases'",
                    search_type
                )),
            })
            .join()
            .unwrap_or_else(|e| error_result(&format!("Thread panicked: {:?}", e)));

            result
        })
    }

    /// Main handler for STDIO/TCP transport.
    pub fn handle_stdio(params: MbRecordingParams) -> BoxFuture<'static, CallToolResult> {
        Box::pin(async move {
            let search_type = params.search_type.clone();
            let query = params.query.clone();
            let limit = validate_limit(params.limit);

            let result = tokio::task::spawn_blocking(move || match search_type.as_str() {
                "recording" => Self::search_recordings(&query, limit),
                "recording_releases" => Self::search_recording_releases(&query, limit),
                _ => error_result(&format!(
                    "Unknown search type: {}. Use 'recording' or 'recording_releases'",
                    search_type
                )),
            })
            .await
            .unwrap_or_else(|e| error_result(&format!("Task failed: {:?}", e)));

            result
        })
    }

    /// Search for recordings by title or MBID.
    pub fn search_recordings(query: &str, limit: usize) -> CallToolResult {
        info!("Searching for recordings matching: {}", query);

        // If the query is a MusicBrainz ID (MBID), fetch the recording directly.
        if is_mbid(query) {
            match Recording::fetch()
                .id(query)
                .with_artists()
                .with_releases()
                .with_genres()
                .execute()
            {
                Ok(recording) => {
                    let artist = get_artist_name(&recording.artist_credit);
                    let duration = recording
                        .length
                        .map(|l| format_duration(l as u64))
                        .unwrap_or_else(|| "--:--".to_string());

                    let mut output = String::new();

                    output.push_str("--- MusicBrainz Recording Details ---\n");
                    output.push_str(&format!("**Title:** {}\n", recording.title));
                    output.push_str(&format!("**Artist(s):** {}\n", artist));
                    output.push_str(&format!("**⏱Duration:** {}\n", duration));
                    output.push_str(&format!("**Recording MBID:** {}\n", recording.id));

                    if let Some(ref disambiguation) = recording.disambiguation {
                        if !disambiguation.is_empty() {
                            output.push_str(&format!(
                                "\n*Note de désambiguïsation: {}*\n",
                                disambiguation
                            ));
                        }
                    }

                    if let Some(artists) = &recording.artist_credit {
                        output.push_str("\n### Artist MBIDs\n");
                        for artist_info in artists {
                            output.push_str(&format!(
                                "- {}: {}\n",
                                artist_info.name, artist_info.artist.id
                            ));
                        }
                    }

                    if let Some(releases) = &recording.releases {
                        if !releases.is_empty() {
                            output.push_str(&format!(
                                "\n### Found on {} Release(s)\n",
                                releases.len()
                            ));

                            for release in releases.iter() {
                                let country = release.country.as_deref().unwrap_or("?");
                                let year = release
                                    .date
                                    .as_ref()
                                    .map(|d| {
                                        d.0.split('-').next().unwrap_or("Year N/A").to_string()
                                    })
                                    .unwrap_or_else(|| "Year N/A".to_string());

                                output.push_str(&format!(
                                    "• {} ({} / {}) - MBID: {}\n",
                                    release.title, country, year, release.id
                                ));
                            }
                        }
                    }

                    if let Some(genres) = &recording.genres {
                        if !genres.is_empty() {
                            output.push_str("\n### Genres\n");
                            let genre_names: Vec<String> =
                                genres.iter().map(|g| g.name.clone()).collect();
                            output.push_str(&format!("- {}\n", genre_names.join(", ")));
                        }
                    }
                    success_result(output)
                }
                Err(e) => {
                    error!("Failed to fetch recording by MBID: {:?}", e);
                    error_result(&format!("Failed to fetch recording: {}", e))
                }
            }
        } else {
            let search_query = RecordingSearchQuery::query_builder()
                .recording(query)
                .build();

            let search_result = Recording::search(search_query).execute();

            match search_result {
                Ok(result) => {
                    let recordings: Vec<_> = result.entities.into_iter().take(limit).collect();
                    if recordings.is_empty() {
                        return error_result(&format!("No recordings found for query: {}", query));
                    }

                    let mut output = format!("Found {} recordings:\n\n", recordings.len());
                    for (i, recording) in recordings.iter().enumerate() {
                        let artist = get_artist_name(&recording.artist_credit);
                        let duration = recording
                            .length
                            .map(|l| format_duration(l as u64))
                            .unwrap_or_else(|| "--:--".to_string());

                        output.push_str(&format!(
                            "{}. **{}** by {} [{}]\n   MBID: {}\n",
                            i + 1,
                            recording.title,
                            artist,
                            duration,
                            recording.id,
                        ));
                        if let Some(ref disambiguation) = recording.disambiguation {
                            if !disambiguation.is_empty() {
                                output.push_str(&format!("   Note: {}\n", disambiguation));
                            }
                        }
                        output.push('\n');
                    }

                    success_result(output)
                }
                Err(e) => {
                    error!("Recording search failed: {:?}", e);
                    error_result(&format!("Recording search failed: {}", e))
                }
            }
        }
    }

    /// Find all releases containing a specific recording.
    pub fn search_recording_releases(query: &str, limit: usize) -> CallToolResult {
        info!("Finding releases containing recording: {}", query);

        // Get the recording MBID
        let recording_id = if is_mbid(query) {
            query.to_string()
        } else {
            // Search for recording first
            let search_query = RecordingSearchQuery::query_builder()
                .recording(query)
                .build();
            match Recording::search(search_query).execute() {
                Ok(result) => {
                    if let Some(recording) = result.entities.first() {
                        debug!("Found recording: {} ({})", recording.title, recording.id);
                        recording.id.clone()
                    } else {
                        return error_result(&format!("No recording found matching: {}", query));
                    }
                }
                Err(e) => {
                    error!("Recording lookup failed: {:?}", e);
                    return error_result(&format!("Recording lookup failed: {}", e));
                }
            }
        };

        // Fetch recording with releases
        match Recording::fetch()
            .id(&recording_id)
            .with_releases()
            .execute()
        {
            Ok(recording) => {
                let artist = get_artist_name(&recording.artist_credit);
                let duration = recording
                    .length
                    .map(|l| format_duration(l as u64))
                    .unwrap_or_else(|| "--:--".to_string());

                let mut output = format!(
                    "**{}** by {} [{}]\nRecording MBID: {}\n\n",
                    recording.title, artist, duration, recording.id
                );

                if let Some(releases) = &recording.releases {
                    output.push_str(&format!(
                        "Found on {} releases:\n\n",
                        releases.len().min(limit)
                    ));
                    for (i, release) in releases.iter().take(limit).enumerate() {
                        let release_artist = get_artist_name(&release.artist_credit);
                        let date = release
                            .date
                            .as_ref()
                            .map(|d| format_date(&d.0))
                            .unwrap_or_else(|| "Unknown".to_string());

                        output.push_str(&format!(
                            "{}. **{}** by {} ({})\n   MBID: {}\n",
                            i + 1,
                            release.title,
                            release_artist,
                            date,
                            release.id,
                        ));
                        if let Some(country) = &release.country {
                            output.push_str(&format!("   Country: {}\n", country));
                        }
                        output.push('\n');
                    }
                } else {
                    output.push_str("No releases found containing this recording.\n");
                }

                success_result(output)
            }
            Err(e) => {
                error!("Failed to fetch recording releases: {:?}", e);
                error_result(&format!("Failed to fetch recording releases: {}", e))
            }
        }
    }
}

impl Default for MbRecordingTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::RawContent;

    #[test]
    fn test_recording_params_default_limit() {
        let json = r#"{"search_type": "recording", "query": "Smells Like Teen Spirit"}"#;
        let params: MbRecordingParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 10);
    }

    // Integration tests (require network, run with: cargo test -- --ignored)
    #[ignore]
    #[test]
    fn test_search_recordings() {
        let result = MbRecordingTool::search_recordings("Paranoid Android", 5);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
        let content = &result.content[0];
        if let RawContent::Text(text) = &content.raw {
            assert!(
                text.text.contains("Paranoid Android"),
                "Expected 'Paranoid Android' in result"
            );
        }
    }

    #[ignore]
    #[test]
    fn test_search_recordings_by_id() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        // Specific recording MBID
        let result = MbRecordingTool::search_recordings("3a909079-a42a-4642-b06f-398bf91f34f4", 5);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
        let content = &result.content[0];
        if let RawContent::Text(text) = &content.raw {
            assert!(
                text.text.contains("3a909079-a42a-4642-b06f-398bf91f34f4") || text.text.len() > 0,
                "Expected non-empty result for MBID"
            );
        }
    }

    #[ignore]
    #[test]
    fn test_search_recording_releases_() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        // Paranoid Android recording MBID
        // Also test searching releases by recording name
        let result = MbRecordingTool::search_recording_releases("Paranoid Android", 10);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
    }

    #[ignore]
    #[test]
    fn test_search_recording_releases_by_id() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        // Paranoid Android recording MBID
        let result =
            MbRecordingTool::search_recording_releases("8b8a07f6-53a6-4025-acb7-d30c7e29fce6", 10);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
    }
}
