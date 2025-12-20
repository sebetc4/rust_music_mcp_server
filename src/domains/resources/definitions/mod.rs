//! Resource definitions module.
//!
//! Each resource is defined in its own file with:
//! - URI and metadata
//! - Content provider
//!
//! ## Adding a New Resource
//!
//! 1. Create a new file (e.g., `my_resource.rs`)
//! 2. Implement the `ResourceDefinition` trait
//! 3. Export it here
//! 4. Register in `registry.rs`

use rmcp::model::ResourceContents;

use super::service::ResourceContent;

/// Trait for resource definitions.
///
/// Each resource must implement this trait to provide its metadata and content.
pub trait ResourceDefinition {
    /// The unique URI of the resource.
    const URI: &'static str;

    /// The display name of the resource.
    const NAME: &'static str;

    /// A description of the resource.
    const DESCRIPTION: &'static str;

    /// The MIME type of the resource content.
    const MIME_TYPE: &'static str;

    /// Get the content for this resource.
    fn content() -> ResourceContent;
}

/// Trait for resources that provide dynamic content.
pub trait DynamicResourceProvider {
    /// Resolve the dynamic content.
    fn resolve(uri: &str, base_path: Option<&str>) -> Result<ResourceContents, String>;
}
