//! MusicBrainz Advanced search tool.
//!
//! This tool provides advanced search capabilities across different entity types
//! in MusicBrainz using the query builder pattern.

use futures::FutureExt;
use futures::future::BoxFuture;
use musicbrainz_rs::{
    Search,
    entity::artist::{Artist, ArtistSearchQuery},
    entity::label::{Label, LabelSearchQuery},
    entity::recording::{Recording, RecordingSearchQuery},
    entity::release::{Release, ReleaseSearchQuery},
    entity::release_group::{ReleaseGroup, ReleaseGroupSearchQuery},
    entity::work::{Work, WorkSearchQuery},
};
use rmcp::{
    ErrorData as McpError,
    handler::server::tool::{ToolCallContext, ToolRoute, cached_schema_for_type},
    model::{CallToolResult, Tool},
};
use schemars::JsonSchema;
use serde::Deserialize;
use tracing::{error, info};

use super::common::{
    default_limit, error_result, extract_year, format_duration, get_artist_name, success_result,
    validate_limit,
};

/// Supported entity types for advanced search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityType {
    Artist,
    Release,
    ReleaseGroup,
    Recording,
    Work,
    Label,
}

impl EntityType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "artist" => Some(Self::Artist),
            "release" => Some(Self::Release),
            "release_group" | "release-group" | "releasegroup" => Some(Self::ReleaseGroup),
            "recording" => Some(Self::Recording),
            "work" => Some(Self::Work),
            "label" => Some(Self::Label),
            _ => None,
        }
    }
}

/// Parameters for advanced search operations.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MbAdvancedSearchParams {
    /// The entity type to search for.
    /// Supported: "artist", "release", "release_group", "recording", "work", "label"
    #[schemars(
        description = "Entity type: artist, release, release_group, recording, work, label"
    )]
    pub entity: String,

    /// The search query string (uses MusicBrainz search syntax).
    #[schemars(description = "Search query string")]
    pub query: String,

    /// Maximum number of results to return (default: 10, max: 100).
    #[schemars(description = "Maximum number of results (default: 10, max: 100)")]
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// MusicBrainz Advanced Search Tool implementation.
#[derive(Debug, Clone)]
pub struct MbAdvancedSearchTool;

impl MbAdvancedSearchTool {
    /// Tool name as registered in MCP.
    pub const NAME: &'static str = "mb_advanced_search";

    /// Tool description shown to clients.
    pub const DESCRIPTION: &'static str = "Advanced MusicBrainz search across multiple entity types: artists, releases, release groups, recordings, works, and labels.";

    pub fn new() -> Self {
        Self
    }

    /// Execute the tool logic (for STDIO/TCP transport via rmcp).
    pub fn execute(params: &MbAdvancedSearchParams) -> CallToolResult {
        let entity = params.entity.clone();
        let query = params.query.clone();
        let limit = validate_limit(params.limit);

        Self::advanced_search(&entity, &query, limit)
    }

    /// HTTP handler for this tool (for HTTP transport).
    #[cfg(feature = "http")]
    pub fn http_handler(arguments: serde_json::Value) -> Result<serde_json::Value, String> {
        let entity = arguments
            .get("entity")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid 'entity' parameter".to_string())?
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

        let params = MbAdvancedSearchParams {
            entity,
            query,
            limit,
        };

        // Use std::thread::spawn to avoid nested runtime panic.
        // musicbrainz_rs uses reqwest::blocking which creates its own runtime.
        let handle = std::thread::spawn(move || Self::execute(&params));

        let result = handle
            .join()
            .map_err(|_| "Thread panicked during advanced search".to_string())?;

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
            input_schema: cached_schema_for_type::<MbAdvancedSearchParams>(),
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
                let params: MbAdvancedSearchParams =
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
    pub fn handle_http(params: MbAdvancedSearchParams) -> BoxFuture<'static, CallToolResult> {
        Box::pin(async move {
            let entity = params.entity.clone();
            let query = params.query.clone();
            let limit = validate_limit(params.limit);

            let result = std::thread::spawn(move || Self::advanced_search(&entity, &query, limit))
                .join()
                .unwrap_or_else(|e| error_result(&format!("Thread panicked: {:?}", e)));

            result
        })
    }

    /// Main handler for STDIO/TCP transport.
    pub fn handle_stdio(params: MbAdvancedSearchParams) -> BoxFuture<'static, CallToolResult> {
        Box::pin(async move {
            let entity = params.entity.clone();
            let query = params.query.clone();
            let limit = validate_limit(params.limit);

            let result =
                tokio::task::spawn_blocking(move || Self::advanced_search(&entity, &query, limit))
                    .await
                    .unwrap_or_else(|e| error_result(&format!("Task failed: {:?}", e)));

            result
        })
    }

    /// Perform advanced search with entity type selection.
    pub fn advanced_search(entity: &str, query: &str, limit: usize) -> CallToolResult {
        let entity_type = match EntityType::from_str(entity) {
            Some(et) => et,
            None => {
                return error_result(&format!(
                    "Invalid entity type: '{}'. Supported: artist, release, release_group, recording, work, label",
                    entity
                ));
            }
        };

        info!("Advanced search: entity={:?}, query={}", entity_type, query);

        match entity_type {
            EntityType::Artist => Self::search_artists(query, limit),
            EntityType::Release => Self::search_releases(query, limit),
            EntityType::ReleaseGroup => Self::search_release_groups(query, limit),
            EntityType::Recording => Self::search_recordings(query, limit),
            EntityType::Work => Self::search_works(query, limit),
            EntityType::Label => Self::search_labels(query, limit),
        }
    }

    fn search_artists(query: &str, limit: usize) -> CallToolResult {
        let search_query = ArtistSearchQuery::query_builder().artist(query).build();
        let search_result = Artist::search(search_query).execute();

        match search_result {
            Ok(result) => {
                let artists: Vec<_> = result.entities.into_iter().take(limit).collect();
                if artists.is_empty() {
                    return error_result(&format!("No artists found for query: {}", query));
                }

                let mut output = format!("Found {} artists:\n\n", artists.len());
                for (i, artist) in artists.iter().enumerate() {
                    output.push_str(&format!(
                        "{}. **{}**\n   MBID: {}\n   Country: {}\n",
                        i + 1,
                        artist.name,
                        artist.id,
                        artist
                            .country
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string()),
                    ));
                    if !artist.disambiguation.is_empty() {
                        output.push_str(&format!("   Note: {}\n", artist.disambiguation));
                    }
                    output.push('\n');
                }

                success_result(output)
            }
            Err(e) => {
                error!("Artist search failed: {:?}", e);
                error_result(&format!("Artist search failed: {}", e))
            }
        }
    }

    fn search_releases(query: &str, limit: usize) -> CallToolResult {
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

    fn search_release_groups(query: &str, limit: usize) -> CallToolResult {
        let search_query = ReleaseGroupSearchQuery::query_builder()
            .release_group(query)
            .build();
        let search_result = ReleaseGroup::search(search_query).execute();

        match search_result {
            Ok(result) => {
                let groups: Vec<_> = result.entities.into_iter().take(limit).collect();
                if groups.is_empty() {
                    return error_result(&format!("No release groups found for query: {}", query));
                }

                let mut output = format!("Found {} release groups:\n\n", groups.len());
                for (i, rg) in groups.iter().enumerate() {
                    let artist = get_artist_name(&rg.artist_credit);
                    let year = rg
                        .first_release_date
                        .as_ref()
                        .and_then(|d| extract_year(&d.0))
                        .unwrap_or_else(|| "Unknown".to_string());

                    output.push_str(&format!(
                        "{}. **{}** by {} ({})\n   MBID: {}\n",
                        i + 1,
                        rg.title,
                        artist,
                        year,
                        rg.id,
                    ));
                    output.push('\n');
                }

                success_result(output)
            }
            Err(e) => {
                error!("Release group search failed: {:?}", e);
                error_result(&format!("Release group search failed: {}", e))
            }
        }
    }

    fn search_recordings(query: &str, limit: usize) -> CallToolResult {
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

    fn search_works(query: &str, limit: usize) -> CallToolResult {
        let search_query = WorkSearchQuery::query_builder().work(query).build();
        let search_result = Work::search(search_query).execute();

        match search_result {
            Ok(result) => {
                let works: Vec<_> = result.entities.into_iter().take(limit).collect();
                if works.is_empty() {
                    return error_result(&format!("No works found for query: {}", query));
                }

                let mut output = format!("Found {} works:\n\n", works.len());
                for (i, work) in works.iter().enumerate() {
                    output.push_str(&format!(
                        "{}. **{}**\n   MBID: {}\n",
                        i + 1,
                        work.title,
                        work.id,
                    ));
                    if let Some(ref disambiguation) = work.disambiguation {
                        if !disambiguation.is_empty() {
                            output.push_str(&format!("   Note: {}\n", disambiguation));
                        }
                    }
                    output.push('\n');
                }

                success_result(output)
            }
            Err(e) => {
                error!("Work search failed: {:?}", e);
                error_result(&format!("Work search failed: {}", e))
            }
        }
    }

    fn search_labels(query: &str, limit: usize) -> CallToolResult {
        let search_query = LabelSearchQuery::query_builder().label(query).build();
        let search_result = Label::search(search_query).execute();

        match search_result {
            Ok(result) => {
                let labels: Vec<_> = result.entities.into_iter().take(limit).collect();
                if labels.is_empty() {
                    return error_result(&format!("No labels found for query: {}", query));
                }

                let mut output = format!("Found {} labels:\n\n", labels.len());
                for (i, label) in labels.iter().enumerate() {
                    output.push_str(&format!(
                        "{}. **{}**\n   MBID: {}\n   Country: {}\n",
                        i + 1,
                        label.name,
                        label.id,
                        label
                            .country
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string()),
                    ));
                    if let Some(ref disambiguation) = label.disambiguation {
                        if !disambiguation.is_empty() {
                            output.push_str(&format!("   Note: {}\n", disambiguation));
                        }
                    }
                    output.push('\n');
                }

                success_result(output)
            }
            Err(e) => {
                error!("Label search failed: {:?}", e);
                error_result(&format!("Label search failed: {}", e))
            }
        }
    }
}

impl Default for MbAdvancedSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::RawContent;

    #[test]
    fn test_entity_type_parsing() {
        assert_eq!(EntityType::from_str("artist"), Some(EntityType::Artist));
        assert_eq!(EntityType::from_str("ARTIST"), Some(EntityType::Artist));
        assert_eq!(EntityType::from_str("release"), Some(EntityType::Release));
        assert_eq!(
            EntityType::from_str("release_group"),
            Some(EntityType::ReleaseGroup)
        );
        assert_eq!(
            EntityType::from_str("release-group"),
            Some(EntityType::ReleaseGroup)
        );
        assert_eq!(
            EntityType::from_str("releasegroup"),
            Some(EntityType::ReleaseGroup)
        );
        assert_eq!(
            EntityType::from_str("recording"),
            Some(EntityType::Recording)
        );
        assert_eq!(EntityType::from_str("work"), Some(EntityType::Work));
        assert_eq!(EntityType::from_str("label"), Some(EntityType::Label));
        assert_eq!(EntityType::from_str("unknown"), None);
    }

    #[test]
    fn test_advanced_search_params_default_limit() {
        let json = r#"{"entity": "artist", "query": "Nirvana"}"#;
        let params: MbAdvancedSearchParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 10);
    }

    // Integration tests (require network, run with: cargo test -- --ignored)
    #[ignore]
    #[test]
    fn test_advanced_search_artist() {
        let result = MbAdvancedSearchTool::advanced_search("artist", "Nirvana", 5);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
        let content = &result.content[0];
        if let RawContent::Text(text) = &content.raw {
            assert!(
                text.text.contains("Nirvana"),
                "Expected 'Nirvana' in result"
            );
        }
    }

    #[ignore]
    #[test]
    fn test_advanced_search_release() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let result = MbAdvancedSearchTool::advanced_search("release", "OK Computer", 5);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
    }

    #[ignore]
    #[test]
    fn test_advanced_search_release_group() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let result = MbAdvancedSearchTool::advanced_search("release_group", "OK Computer", 5);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
    }

    #[ignore]
    #[test]
    fn test_advanced_search_recording() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let result = MbAdvancedSearchTool::advanced_search("recording", "Paranoid Android", 5);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
    }

    #[ignore]
    #[test]
    fn test_advanced_search_work() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let result = MbAdvancedSearchTool::advanced_search("work", "Bohemian Rhapsody", 5);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
    }

    #[ignore]
    #[test]
    fn test_advanced_search_label() {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let result = MbAdvancedSearchTool::advanced_search("label", "Sony", 5);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
    }

    #[ignore]
    #[test]
    fn test_advanced_search_invalid_entity() {
        let result = MbAdvancedSearchTool::advanced_search("invalid", "test", 5);
        assert!(
            result.is_error.unwrap_or(false),
            "Expected error for invalid entity"
        );
    }
}
