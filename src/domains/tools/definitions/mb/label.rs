//! MusicBrainz Label search tool.
//!
//! This tool provides functionality to search for labels (record labels/publishers).
//! Labels represent the companies or organizations that publish music releases.

use futures::FutureExt;
use futures::future::BoxFuture;
use musicbrainz_rs::{
    Search,
    entity::label::{Label, LabelSearchQuery},
};
use rmcp::{
    ErrorData as McpError,
    handler::server::tool::{ToolCallContext, ToolRoute, cached_schema_for_type},
    model::{CallToolResult, Tool},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use super::common::{
    default_limit, error_result, structured_result, validate_limit,
};

/// Parameters for label search operations.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MbLabelParams {
    /// The search query string (label name).
    #[schemars(description = "Search query (label name)")]
    pub query: String,

    /// Maximum number of results to return (default: 10, max: 100).
    #[schemars(description = "Maximum number of results (default: 10, max: 100)")]
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// Structured output for label search results.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct LabelSearchResult {
    pub labels: Vec<LabelInfo>,
    pub total_count: usize,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct LabelInfo {
    pub name: String,
    pub mbid: String,
    pub label_type: Option<String>,
    pub country: Option<String>,
    pub disambiguation: Option<String>,
    pub label_code: Option<i32>,
}

/// MusicBrainz Label Search Tool implementation.
#[derive(Debug, Clone)]
pub struct MbLabelTool;

impl MbLabelTool {
    /// Tool name as registered in MCP.
    pub const NAME: &'static str = "mb_label_search";

    /// Tool description shown to clients.
    pub const DESCRIPTION: &'static str = "Search for labels (record labels/publishers) in MusicBrainz. Labels represent the companies or organizations that publish music releases. Returns structured data with MBIDs, label types, countries, label codes, and disambiguation info.";

    pub fn new() -> Self {
        Self
    }

    /// Execute the tool logic (for STDIO/TCP transport via rmcp).
    pub fn execute(params: &MbLabelParams) -> CallToolResult {
        let query = params.query.clone();
        let limit = validate_limit(params.limit);

        Self::search_labels(&query, limit)
    }

    /// HTTP handler for this tool (for HTTP transport).
    #[cfg(feature = "http")]
    pub fn http_handler(arguments: serde_json::Value) -> Result<serde_json::Value, String> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid 'query' parameter".to_string())?
            .to_string();

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let params = MbLabelParams {
            query,
            limit,
        };

        // Use std::thread::spawn to avoid nested runtime panic.
        // musicbrainz_rs uses reqwest::blocking which creates its own runtime.
        let handle = std::thread::spawn(move || Self::execute(&params));

        let result = handle
            .join()
            .map_err(|_| "Thread panicked during label search".to_string())?;

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
            input_schema: cached_schema_for_type::<MbLabelParams>(),
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
                let params: MbLabelParams =
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
    pub fn handle_http(params: MbLabelParams) -> BoxFuture<'static, CallToolResult> {
        Box::pin(async move {
            let query = params.query.clone();
            let limit = validate_limit(params.limit);

            let result = std::thread::spawn(move || Self::search_labels(&query, limit))
                .join()
                .unwrap_or_else(|e| error_result(&format!("Thread panicked: {:?}", e)));

            result
        })
    }

    /// Main handler for STDIO/TCP transport.
    pub fn handle_stdio(params: MbLabelParams) -> BoxFuture<'static, CallToolResult> {
        Box::pin(async move {
            let query = params.query.clone();
            let limit = validate_limit(params.limit);

            let result = tokio::task::spawn_blocking(move || Self::search_labels(&query, limit))
                .await
                .unwrap_or_else(|e| error_result(&format!("Task failed: {:?}", e)));

            result
        })
    }

    /// Search for labels by name.
    pub fn search_labels(query: &str, limit: usize) -> CallToolResult {
        info!("Searching for labels matching: {}", query);

        let search_query = LabelSearchQuery::query_builder().label(query).build();
        let search_result = Label::search(search_query).execute();

        match search_result {
            Ok(result) => {
                let labels: Vec<_> = result.entities.into_iter().take(limit).collect();
                if labels.is_empty() {
                    return error_result(&format!("No labels found for query: {}", query));
                }

                let count = labels.len();
                let label_infos: Vec<LabelInfo> = labels
                    .into_iter()
                    .map(|l| LabelInfo {
                        name: l.name,
                        mbid: l.id,
                        label_type: l.label_type.map(|t| format!("{:?}", t)),
                        country: l.country,
                        disambiguation: l.disambiguation.filter(|d| !d.is_empty()),
                        label_code: l.label_code.map(|c| c as i32),
                    })
                    .collect();

                let structured_data = LabelSearchResult {
                    labels: label_infos,
                    total_count: count,
                    query: query.to_string(),
                };

                let summary = format!("Found {} label(s) matching '{}'", count, query);
                structured_result(summary, structured_data)
            }
            Err(e) => {
                error!("Label search failed: {:?}", e);
                error_result(&format!("Label search failed: {}", e))
            }
        }
    }
}

impl Default for MbLabelTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::RawContent;

    #[test]
    fn test_label_params_default_limit() {
        let json = r#"{"query": "Sony Music"}"#;
        let params: MbLabelParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 10);
    }

    // Integration tests (require network, run with: cargo test -- --ignored)
    #[ignore]
    #[test]
    fn test_search_labels() {
        let result = MbLabelTool::search_labels("Sony", 5);
        assert!(
            !result.is_error.unwrap_or(true),
            "Expected success but got error"
        );
        let content = &result.content[0];
        if let RawContent::Text(text) = &content.raw {
            assert!(
                text.text.contains("Sony") || text.text.contains("label"),
                "Expected label-related content in result"
            );
        }
    }
}
