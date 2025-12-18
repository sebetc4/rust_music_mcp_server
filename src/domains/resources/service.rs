//! Resource service implementation.
//!
//! The ResourceService manages resource discovery and access.
//! It maintains a registry of available resources and handles read requests.
//!
//! Resources are defined in `definitions/` and registered via `registry.rs`.
//! Adding a new resource does NOT require modifying this file.

use rmcp::model::{ReadResourceResult, Resource, ResourceContents, ResourceTemplate};
use std::collections::HashMap;
use tracing::info;

use super::error::ResourceError;
use super::registry::{get_all_resource_templates, get_all_resources};
use crate::core::config::ResourcesConfig;

/// Service for managing and accessing resources.
///
/// This service maintains a registry of available resources and handles
/// resource listing and reading operations.
pub struct ResourceService {
    /// Configuration for the resources domain.
    config: ResourcesConfig,

    /// Registry of available resources.
    /// Key: resource URI, Value: resource metadata
    resources: HashMap<String, ResourceEntry>,

    /// Resource templates for parameterized resources.
    templates: Vec<ResourceTemplate>,
}

/// An entry in the resource registry.
#[derive(Debug, Clone)]
pub struct ResourceEntry {
    /// The resource metadata.
    pub resource: Resource,

    /// The content provider for this resource.
    pub content: ResourceContent,
}

/// Different types of resource content.
#[derive(Debug, Clone)]
pub enum ResourceContent {
    /// Static text content.
    Text(String),

    /// Static binary content (base64 encoded).
    Binary(Vec<u8>),

    /// Dynamic content that requires computation.
    Dynamic(DynamicResourceType),
}

/// Types of dynamic resources.
#[derive(Debug, Clone)]
pub enum DynamicResourceType {
    /// System information resource.
    SystemInfo,

    /// File system resource (path relative to base_path).
    File(String),

    /// Custom dynamic resource.
    Custom(String),
}

impl ResourceService {
    /// Create a new ResourceService with the given configuration.
    pub fn new(config: ResourcesConfig) -> Self {
        info!("Initializing ResourceService");

        let mut service = Self {
            config,
            resources: HashMap::new(),
            templates: Vec::new(),
        };

        // Register all resources and templates from registry
        service.register_from_registry();
        service.register_templates_from_registry();

        service
    }

    /// Register all resources from the registry.
    fn register_from_registry(&mut self) {
        info!("Registering resources from registry");
        for entry in get_all_resources() {
            self.register_resource(entry);
        }
    }

    /// Register all resource templates from the registry.
    fn register_templates_from_registry(&mut self) {
        info!("Registering resource templates from registry");
        self.templates = get_all_resource_templates();
    }

    /// Register a resource.
    pub fn register_resource(&mut self, entry: ResourceEntry) {
        info!("Registering resource: {}", entry.resource.raw.uri);
        self.resources
            .insert(entry.resource.raw.uri.to_string(), entry);
    }

    /// List all available resources.
    pub async fn list_resources(&self) -> Vec<Resource> {
        self.resources
            .values()
            .map(|entry| entry.resource.clone())
            .collect()
    }

    /// List all available resource templates.
    pub async fn list_resource_templates(&self) -> Vec<ResourceTemplate> {
        self.templates.clone()
    }

    /// Read a resource by URI.
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult, ResourceError> {
        let entry = self
            .resources
            .get(uri)
            .ok_or_else(|| ResourceError::not_found(uri))?;

        let content = match &entry.content {
            ResourceContent::Text(text) => ResourceContents::text(text, uri),
            ResourceContent::Binary(data) => ResourceContents::BlobResourceContents {
                uri: uri.to_string(),
                mime_type: entry.resource.raw.mime_type.clone(),
                blob: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data),
                meta: None,
            },
            ResourceContent::Dynamic(dynamic_type) => {
                self.resolve_dynamic_content(uri, dynamic_type)?
            }
        };

        Ok(ReadResourceResult {
            contents: vec![content],
        })
    }

    /// Resolve dynamic resource content.
    fn resolve_dynamic_content(
        &self,
        uri: &str,
        dynamic_type: &DynamicResourceType,
    ) -> Result<ResourceContents, ResourceError> {
        match dynamic_type {
            DynamicResourceType::SystemInfo => {
                let info = serde_json::json!({
                    "server": "MCP Server Template",
                    "version": env!("CARGO_PKG_VERSION"),
                    "base_path": self.config.base_path,
                });

                Ok(ResourceContents::text(
                    serde_json::to_string_pretty(&info)
                        .map_err(|e| ResourceError::internal(e.to_string()))?,
                    uri,
                ))
            }
            DynamicResourceType::File(path) => {
                let full_path = if let Some(base) = &self.config.base_path {
                    format!("{}/{}", base, path)
                } else {
                    path.clone()
                };

                let content = std::fs::read_to_string(&full_path)?;

                Ok(ResourceContents::text(content, uri))
            }
            DynamicResourceType::Custom(identifier) => Ok(ResourceContents::text(
                format!("Custom resource: {}", identifier),
                uri,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_service_creation() {
        let config = ResourcesConfig::default();
        let service = ResourceService::new(config);

        let resources = service.list_resources().await;
        assert!(!resources.is_empty());
    }

    #[tokio::test]
    async fn test_read_existing_resource() {
        let config = ResourcesConfig::default();
        let service = ResourceService::new(config);

        let result = service.read_resource("mcp://server/docs/readme").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_read_nonexistent_resource() {
        let config = ResourcesConfig::default();
        let service = ResourceService::new(config);

        let result = service.read_resource("mcp://server/nonexistent").await;
        assert!(result.is_err());
    }
}
