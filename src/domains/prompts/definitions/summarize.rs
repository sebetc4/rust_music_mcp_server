//! Summarize prompt definition.

use super::PromptDefinition;
use rmcp::model::PromptArgument;

/// Summarize text or content.
pub struct SummarizePrompt;

impl PromptDefinition for SummarizePrompt {
    const NAME: &'static str = "summarize";
    const DESCRIPTION: &'static str = "Summarize text or content";

    fn template() -> &'static str {
        r#"Please summarize the following content{{#if length}} ({{length}} summary){{/if}}:

{{content}}"#
    }

    fn arguments() -> Vec<PromptArgument> {
        vec![
            PromptArgument {
                name: "content".to_string(),
                title: None,
                description: Some("The content to summarize".to_string()),
                required: Some(true),
            },
            PromptArgument {
                name: "length".to_string(),
                title: None,
                description: Some("Desired length: brief, medium, or detailed".to_string()),
                required: Some(false),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_prompt_metadata() {
        assert_eq!(SummarizePrompt::NAME, "summarize");
        assert!(!SummarizePrompt::DESCRIPTION.is_empty());

        let args = SummarizePrompt::arguments();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].name, "content");
        assert_eq!(args[0].required, Some(true));
    }
}
