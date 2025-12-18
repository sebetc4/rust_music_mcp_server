//! Example configuration resource definition.

use super::ResourceDefinition;
use crate::domains::resources::service::ResourceContent;

/// Example configuration resource (static JSON).
pub struct ConfigExampleResource;

impl ResourceDefinition for ConfigExampleResource {
    const URI: &'static str = "mcp://server/config/example";
    const NAME: &'static str = "Example Configuration";
    const DESCRIPTION: &'static str = "An example configuration resource";
    const MIME_TYPE: &'static str = "application/json";

    fn content() -> ResourceContent {
        ResourceContent::Text(
            serde_json::json!({
                "example": true,
                "settings": {
                    "debug": false,
                    "max_connections": 100
                }
            })
            .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_example_metadata() {
        assert_eq!(ConfigExampleResource::URI, "mcp://server/config/example");
        assert_eq!(ConfigExampleResource::MIME_TYPE, "application/json");
    }

    #[test]
    fn test_config_example_content() {
        match ConfigExampleResource::content() {
            ResourceContent::Text(text) => {
                assert!(text.contains("example"));
                assert!(text.contains("settings"));
            }
            _ => panic!("Expected Text content"),
        }
    }
}
