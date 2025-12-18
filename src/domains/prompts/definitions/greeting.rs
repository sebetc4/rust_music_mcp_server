//! Greeting prompt definition.

use super::PromptDefinition;
use rmcp::model::PromptArgument;

/// A customizable greeting prompt.
pub struct GreetingPrompt;

impl PromptDefinition for GreetingPrompt {
    const NAME: &'static str = "greeting";
    const DESCRIPTION: &'static str = "A customizable greeting prompt";

    fn template() -> &'static str {
        "Hello, {{name}}! {{#if style}}(Style: {{style}}){{/if}}"
    }

    fn arguments() -> Vec<PromptArgument> {
        vec![
            PromptArgument {
                name: "name".to_string(),
                title: None,
                description: Some("The name to greet".to_string()),
                required: Some(true),
            },
            PromptArgument {
                name: "style".to_string(),
                title: None,
                description: Some(
                    "The greeting style: formal, casual, or enthusiastic".to_string(),
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
    fn test_greeting_prompt_metadata() {
        assert_eq!(GreetingPrompt::NAME, "greeting");
        assert!(!GreetingPrompt::DESCRIPTION.is_empty());
        assert!(!GreetingPrompt::template().is_empty());

        let args = GreetingPrompt::arguments();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].name, "name");
        assert_eq!(args[0].required, Some(true));
    }
}
