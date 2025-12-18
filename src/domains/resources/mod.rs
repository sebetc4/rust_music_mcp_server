//! Resources domain module.
//!
//! This module handles all resource-related functionality for the MCP server.
//! Resources represent data that can be read by MCP clients, such as files,
//! database records, or API responses.
//!
//! ## Architecture
//!
//! - `definitions/` - Individual resource definitions (one file per resource)
//! - `registry.rs` - Central resource registration
//! - `service.rs` - Resource service for listing and reading
//!
//! ## Adding a New Resource
//!
//! 1. Create a new file in `definitions/` (e.g., `my_resource.rs`)
//! 2. Implement the `ResourceDefinition` trait
//! 3. Export in `definitions/mod.rs`
//! 4. Register in `registry.rs`
//!
//! **No need to modify `service.rs`!**

pub mod definitions;
mod error;
mod handlers;
mod registry;
mod service;

pub use definitions::ResourceDefinition;
pub use error::ResourceError;
pub use handlers::*;
pub use registry::{get_all_resources, resource_uris};
pub use service::{DynamicResourceType, ResourceContent, ResourceEntry, ResourceService};
