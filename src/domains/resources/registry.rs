//! Resource Registry - central registration of all resources.
//!
//! This module provides dynamic resource registration without modifying service.rs.
//! When adding a new resource:
//! 1. Create the resource file in `definitions/`
//! 2. Export it in `definitions/mod.rs`
//! 3. Register it here in `get_all_resources()`

use rmcp::model::{AnnotateAble, RawResource, RawResourceTemplate, ResourceTemplate};

use super::definitions::ResourceDefinition;
use super::service::ResourceEntry;

/// Helper function to create an annotated resource from a definition.
#[allow(unused)]
fn build_resource<R: ResourceDefinition>() -> ResourceEntry {
    let mut raw = RawResource::new(R::URI, R::NAME);
    raw.description = Some(R::DESCRIPTION.to_string());
    raw.mime_type = Some(R::MIME_TYPE.to_string());

    ResourceEntry {
        resource: raw.no_annotation(),
        content: R::content(),
    }
}

/// Get all registered resources as ResourceEntries.
///
/// This is the central place where all resources are registered.
/// When adding a new resource, add it here.
pub fn get_all_resources() -> Vec<ResourceEntry> {
    vec![]
}

/// Get all registered resource templates.
///
/// Resource templates use URI templates (RFC 6570) to describe
/// parameterized resources that clients can fill in.
pub fn get_all_resource_templates() -> Vec<ResourceTemplate> {
    vec![
        // File access template
        RawResourceTemplate {
            uri_template: "file:///{path}".to_string(),
            name: "Project Files".to_string(),
            title: Some("Access Project Files".to_string()),
            description: Some(
                "Read files from the project directory by specifying the path".to_string(),
            ),
            mime_type: Some("application/octet-stream".to_string()),
        }
        .no_annotation(),
        // Configuration template
        RawResourceTemplate {
            uri_template: "config://{section}/{key}".to_string(),
            name: "Configuration Values".to_string(),
            title: Some("Access Configuration".to_string()),
            description: Some("Access configuration values by section and key".to_string()),
            mime_type: Some("application/json".to_string()),
        }
        .no_annotation(),
        // Documentation template
        RawResourceTemplate {
            uri_template: "mcp://server/docs/{document}".to_string(),
            name: "Server Documentation".to_string(),
            title: Some("Server Docs".to_string()),
            description: Some("Access server documentation by document name".to_string()),
            mime_type: Some("text/markdown".to_string()),
        }
        .no_annotation(),
    ]
}

/// Get the list of all resource URIs.
pub fn resource_uris() -> Vec<&'static str> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_resources() {
        let resources = get_all_resources();
        assert_eq!(resources.len(), 3);

        let uris: Vec<_> = resources
            .iter()
            .map(|r| r.resource.raw.uri.as_str())
            .collect();
        assert!(uris.contains(&"mcp://server/info"));
        assert!(uris.contains(&"mcp://server/config/example"));
        assert!(uris.contains(&"mcp://server/docs/readme"));
    }

    #[test]
    fn test_get_all_resource_templates() {
        let templates = get_all_resource_templates();
        assert_eq!(templates.len(), 3);

        let uri_templates: Vec<_> = templates
            .iter()
            .map(|t| t.raw.uri_template.as_str())
            .collect();
        assert!(uri_templates.contains(&"file:///{path}"));
        assert!(uri_templates.contains(&"config://{section}/{key}"));
        assert!(uri_templates.contains(&"mcp://server/docs/{document}"));
    }

    #[test]
    fn test_resource_uris() {
        let uris = resource_uris();
        assert_eq!(uris.len(), 3);
        assert!(uris.contains(&"mcp://server/info"));
    }
}
