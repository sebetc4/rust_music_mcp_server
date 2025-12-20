//! Prompt definitions module.
//!
//! Each prompt is defined in its own file with:
//! - Metadata (name, description, arguments)
//! - Template string
//!
//! ## Adding a New Prompt
//!
//! 1. Create a new file (e.g., `my_prompt.rs`)
//! 2. Implement the `PromptDefinition` trait
//! 3. Export it here
//! 4. Register in `registry.rs`

use rmcp::model::PromptArgument;

/// Trait for prompt definitions.
///
/// Each prompt must implement this trait to provide its metadata and template.
pub trait PromptDefinition {
    /// The unique name of the prompt.
    const NAME: &'static str;

    /// A description of what the prompt does.
    const DESCRIPTION: &'static str;

    /// The template string with {{variable}} placeholders.
    fn template() -> &'static str;

    /// The arguments this prompt accepts.
    fn arguments() -> Vec<PromptArgument>;
}
