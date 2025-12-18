//! Resource handlers module.
//!
//! This module contains utilities and traits for implementing custom
//! resource handlers that can provide dynamic content.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Metadata about a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    /// The URI of the resource.
    pub uri: String,

    /// The name of the resource.
    pub name: String,

    /// A description of the resource.
    pub description: Option<String>,

    /// The MIME type of the resource content.
    pub mime_type: Option<String>,

    /// The size of the resource in bytes (if known).
    pub size: Option<u64>,

    /// When the resource was last modified (as ISO 8601 string).
    pub last_modified: Option<String>,
}

/// The result of reading a resource.
#[derive(Debug, Clone)]
pub enum ResourceReadResult {
    /// Text content.
    Text {
        content: String,
        mime_type: Option<String>,
    },

    /// Binary content.
    Binary {
        content: Vec<u8>,
        mime_type: Option<String>,
    },
}

/// Trait for implementing custom resource handlers.
///
/// Implement this trait when you need complex resource logic that
/// requires dynamic content generation or external data fetching.
#[async_trait]
pub trait ResourceHandler: Send + Sync {
    /// Get the URI pattern this handler matches.
    ///
    /// This can be an exact URI or a pattern (e.g., "mcp://files/*").
    fn uri_pattern(&self) -> &str;

    /// Check if this handler can handle the given URI.
    fn matches(&self, uri: &str) -> bool {
        let pattern = self.uri_pattern();
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            uri.starts_with(prefix)
        } else {
            uri == pattern
        }
    }

    /// List resources matching this handler's pattern.
    async fn list(&self) -> Vec<ResourceMetadata>;

    /// Read the content of a resource.
    async fn read(&self, uri: &str) -> Result<ResourceReadResult, String>;
}

// ============================================================================
// Example: Database resource handler
// ============================================================================

/// A resource handler for database-backed resources.
///
/// This is an example of a custom resource handler that could
/// fetch data from a database.
pub struct DatabaseResourceHandler {
    /// The URI prefix for database resources.
    prefix: String,

    /// Database connection string (placeholder).
    #[allow(dead_code)]
    connection_string: String,
}

impl DatabaseResourceHandler {
    /// Create a new DatabaseResourceHandler.
    pub fn new(prefix: String, connection_string: String) -> Self {
        Self {
            prefix,
            connection_string,
        }
    }
}

#[async_trait]
impl ResourceHandler for DatabaseResourceHandler {
    fn uri_pattern(&self) -> &str {
        &self.prefix
    }

    async fn list(&self) -> Vec<ResourceMetadata> {
        // In a real implementation, this would query the database
        // for available resources.
        vec![ResourceMetadata {
            uri: format!("{}tables", self.prefix),
            name: "Database Tables".to_string(),
            description: Some("List of database tables".to_string()),
            mime_type: Some("application/json".to_string()),
            size: None,
            last_modified: None,
        }]
    }

    async fn read(&self, uri: &str) -> Result<ResourceReadResult, String> {
        // In a real implementation, this would fetch data from the database
        if uri.ends_with("tables") {
            Ok(ResourceReadResult::Text {
                content: serde_json::json!({
                    "tables": ["users", "orders", "products"],
                    "note": "This is example data"
                })
                .to_string(),
                mime_type: Some("application/json".to_string()),
            })
        } else {
            Err(format!("Resource not found: {}", uri))
        }
    }
}

// ============================================================================
// Example: HTTP resource handler
// ============================================================================

/// A resource handler for HTTP-fetched resources.
///
/// This handler can proxy external HTTP resources.
pub struct HttpResourceHandler {
    /// The URI prefix for HTTP resources.
    prefix: String,

    /// Base URL for HTTP requests.
    #[allow(dead_code)]
    base_url: String,
}

impl HttpResourceHandler {
    /// Create a new HttpResourceHandler.
    pub fn new(prefix: String, base_url: String) -> Self {
        Self { prefix, base_url }
    }
}

#[async_trait]
impl ResourceHandler for HttpResourceHandler {
    fn uri_pattern(&self) -> &str {
        &self.prefix
    }

    async fn list(&self) -> Vec<ResourceMetadata> {
        // HTTP resources are typically discovered through other means
        vec![]
    }

    async fn read(&self, uri: &str) -> Result<ResourceReadResult, String> {
        // In a real implementation, this would make an HTTP request
        // to fetch the resource content.
        Ok(ResourceReadResult::Text {
            content: format!("HTTP resource placeholder for: {}", uri),
            mime_type: Some("text/plain".to_string()),
        })
    }
}
