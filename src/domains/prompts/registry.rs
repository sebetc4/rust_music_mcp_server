//! Prompt Registry - central registration of all prompts.
//!
//! This module provides dynamic prompt registration without modifying service.rs.
//! When adding a new prompt:
//! 1. Create the prompt file in `definitions/`
//! 2. Export it in `definitions/mod.rs`
//! 3. Register it here in `register_all_prompts()`

use super::definitions::{
    CodeReviewPrompt, ExplainPrompt, GreetingPrompt, PromptDefinition, SummarizePrompt,
};
use super::templates::PromptTemplate;

/// Build a PromptTemplate from a PromptDefinition.
fn build_template<P: PromptDefinition>() -> PromptTemplate {
    PromptTemplate {
        name: P::NAME.to_string(),
        description: Some(P::DESCRIPTION.to_string()),
        arguments: P::arguments(),
        template: P::template().to_string(),
    }
}

/// Get all registered prompts as PromptTemplates.
///
/// This is the central place where all prompts are registered.
/// When adding a new prompt, add it here.
pub fn get_all_prompts() -> Vec<PromptTemplate> {
    vec![
        build_template::<GreetingPrompt>(),
        build_template::<CodeReviewPrompt>(),
        build_template::<ExplainPrompt>(),
        build_template::<SummarizePrompt>(),
    ]
}

/// Get the list of all prompt names.
pub fn prompt_names() -> Vec<&'static str> {
    vec![
        GreetingPrompt::NAME,
        CodeReviewPrompt::NAME,
        ExplainPrompt::NAME,
        SummarizePrompt::NAME,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_prompts() {
        let prompts = get_all_prompts();
        assert_eq!(prompts.len(), 4);

        let names: Vec<_> = prompts.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"greeting"));
        assert!(names.contains(&"code_review"));
        assert!(names.contains(&"explain"));
        assert!(names.contains(&"summarize"));
    }

    #[test]
    fn test_prompt_names() {
        let names = prompt_names();
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"greeting"));
    }
}
