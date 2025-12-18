//! Documentation readme resource definition.

use super::ResourceDefinition;
use crate::domains::resources::service::ResourceContent;

/// Server documentation resource (static Markdown).
pub struct DocsReadmeResource;

impl ResourceDefinition for DocsReadmeResource {
    const URI: &'static str = "mcp://server/docs/readme";
    const NAME: &'static str = "Server Documentation";
    const DESCRIPTION: &'static str = "Documentation for using this MCP server";
    const MIME_TYPE: &'static str = "text/markdown";

    fn content() -> ResourceContent {
        ResourceContent::Text(DOCUMENTATION.to_string())
    }
}

const DOCUMENTATION: &str = r#"# MCP Server Template

Welcome to the MCP Server Template!

## Available Tools

- `echo`: Echo back a message
- `add`: Add two numbers
- `system_info`: Get system information

## Available Resources

- `mcp://server/info`: Server information
- `mcp://server/config/example`: Example configuration
- `mcp://server/docs/readme`: This documentation

## Available Prompts

- `greeting`: A customizable greeting prompt
- `code_review`: A code review prompt template
- `explain`: Ask for an explanation of a concept
- `summarize`: Summarize text or content
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docs_readme_metadata() {
        assert_eq!(DocsReadmeResource::URI, "mcp://server/docs/readme");
        assert_eq!(DocsReadmeResource::MIME_TYPE, "text/markdown");
    }

    #[test]
    fn test_docs_readme_content() {
        match DocsReadmeResource::content() {
            ResourceContent::Text(text) => {
                assert!(text.contains("MCP Server Template"));
                assert!(text.contains("Available Tools"));
            }
            _ => panic!("Expected Text content"),
        }
    }
}
