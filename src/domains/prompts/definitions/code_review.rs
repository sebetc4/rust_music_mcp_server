//! Code review prompt definition.

use super::PromptDefinition;
use rmcp::model::PromptArgument;

/// A code review prompt template.
pub struct CodeReviewPrompt;

impl PromptDefinition for CodeReviewPrompt {
    const NAME: &'static str = "code_review";
    const DESCRIPTION: &'static str = "A code review prompt template";

    fn template() -> &'static str {
        r#"Please review the following {{language}} code:

```{{language}}
{{code}}
```

{{#if focus}}
Please focus specifically on: {{focus}}
{{else}}
Please provide a comprehensive review covering:
- Code quality and readability
- Potential bugs or issues
- Performance considerations
- Security concerns
- Suggestions for improvement
{{/if}}"#
    }

    fn arguments() -> Vec<PromptArgument> {
        vec![
            PromptArgument {
                name: "language".to_string(),
                title: None,
                description: Some("The programming language of the code".to_string()),
                required: Some(true),
            },
            PromptArgument {
                name: "code".to_string(),
                title: None,
                description: Some("The code to review".to_string()),
                required: Some(true),
            },
            PromptArgument {
                name: "focus".to_string(),
                title: None,
                description: Some(
                    "Specific areas to focus on (e.g., security, performance)".to_string(),
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
    fn test_code_review_prompt_metadata() {
        assert_eq!(CodeReviewPrompt::NAME, "code_review");
        assert!(!CodeReviewPrompt::DESCRIPTION.is_empty());

        let args = CodeReviewPrompt::arguments();
        assert_eq!(args.len(), 3);

        // language and code are required
        assert_eq!(args[0].required, Some(true));
        assert_eq!(args[1].required, Some(true));
        // focus is optional
        assert_eq!(args[2].required, Some(false));
    }
}
