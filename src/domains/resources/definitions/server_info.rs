//! Server info resource definition.

use super::{DynamicResourceProvider, ResourceDefinition};
use crate::domains::resources::service::{DynamicResourceType, ResourceContent};
use rmcp::model::ResourceContents;

/// Server information resource (dynamic).
pub struct ServerInfoResource;

impl ResourceDefinition for ServerInfoResource {
    const URI: &'static str = "mcp://server/info";
    const NAME: &'static str = "Server Information";
    const DESCRIPTION: &'static str = "Information about this MCP server";
    const MIME_TYPE: &'static str = "application/json";

    fn content() -> ResourceContent {
        ResourceContent::Dynamic(DynamicResourceType::SystemInfo)
    }
}

impl DynamicResourceProvider for ServerInfoResource {
    fn resolve(uri: &str, base_path: Option<&str>) -> Result<ResourceContents, String> {
        let info = serde_json::json!({
            "server": "MCP Server Template",
            "version": env!("CARGO_PKG_VERSION"),
            "base_path": base_path,
        });

        Ok(ResourceContents::text(
            serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?,
            uri,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info_metadata() {
        assert_eq!(ServerInfoResource::URI, "mcp://server/info");
        assert_eq!(ServerInfoResource::MIME_TYPE, "application/json");
    }

    #[test]
    fn test_server_info_resolve() {
        let result = ServerInfoResource::resolve("mcp://server/info", None);
        assert!(result.is_ok());
    }
}
