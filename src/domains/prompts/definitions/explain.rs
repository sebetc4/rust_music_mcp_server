//! Explain prompt definition.

use super::PromptDefinition;
use rmcp::model::PromptArgument;

/// Ask for an explanation of a concept.
pub struct ExplainPrompt;

impl PromptDefinition for ExplainPrompt {
    const NAME: &'static str = "explain";
    const DESCRIPTION: &'static str = "Ask for an explanation of a concept";

    fn template() -> &'static str {
        r#"Please explain {{topic}}{{#if level}} for someone with {{level}} knowledge{{/if}}.

Provide:
1. A clear definition
2. Key concepts
3. Practical examples
4. Common use cases"#
    }

    fn arguments() -> Vec<PromptArgument> {
        vec![
            PromptArgument {
                name: "topic".to_string(),
                title: None,
                description: Some("The topic to explain".to_string()),
                required: Some(true),
            },
            PromptArgument {
                name: "level".to_string(),
                title: None,
                description: Some(
                    "The expertise level: beginner, intermediate, or advanced".to_string(),
                ),
                required: Some(false),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_prompt_metadata() {
        assert_eq!(ExplainPrompt::NAME, "explain");
        assert!(!ExplainPrompt::DESCRIPTION.is_empty());

        let args = ExplainPrompt::arguments();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].name, "topic");
        assert_eq!(args[0].required, Some(true));
    }
}
