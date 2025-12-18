//! Prompts domain module.
//!
//! This module handles all prompt-related functionality for the MCP server.
//! Prompts are template messages that can be customized with arguments and
//! used to generate consistent interactions with language models.
//!
//! ## Architecture
//!
//! - `definitions/` - Individual prompt definitions (one file per prompt)
//! - `registry.rs` - Central prompt registration
//! - `service.rs` - Prompt service for listing and rendering
//! - `templates.rs` - Template rendering engine
//!
//! ## Adding a New Prompt
//!
//! 1. Create a new file in `definitions/` (e.g., `my_prompt.rs`)
//! 2. Implement the `PromptDefinition` trait
//! 3. Export in `definitions/mod.rs`
//! 4. Register in `registry.rs`
//!
//! **No need to modify `service.rs`!**

pub mod definitions;
mod error;
mod registry;
mod service;
pub mod templates;

pub use definitions::PromptDefinition;
pub use error::PromptError;
pub use registry::{get_all_prompts, prompt_names};
pub use service::PromptService;
pub use templates::PromptTemplate;
