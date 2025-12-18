//! Prompt templates module.
//!
//! This module contains the PromptTemplate struct and related utilities
//! for defining and rendering prompt templates.

use rmcp::model::PromptArgument;
use std::collections::HashMap;

use super::error::PromptError;

/// A prompt template that can be instantiated with arguments.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// The unique name of the prompt.
    pub name: String,

    /// A description of what the prompt does.
    pub description: Option<String>,

    /// The arguments that this prompt accepts.
    pub arguments: Vec<PromptArgument>,

    /// The template string with placeholders.
    /// Uses a simple {{variable}} syntax for substitution.
    pub template: String,
}

impl PromptTemplate {
    /// Create a new prompt template.
    pub fn new(
        name: impl Into<String>,
        description: Option<String>,
        arguments: Vec<PromptArgument>,
        template: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description,
            arguments,
            template: template.into(),
        }
    }

    /// Render the template with the given arguments.
    ///
    /// This method performs simple variable substitution:
    /// - `{{variable}}` is replaced with the value of `variable`
    /// - `{{#if variable}}content{{/if}}` includes content only if variable is set
    /// - `{{#if variable}}content{{else}}alternative{{/if}}` with else support
    pub fn render(&self, arguments: &HashMap<String, String>) -> Result<String, PromptError> {
        let mut result = self.template.clone();

        // Process conditionals first
        result = self.process_conditionals(&result, arguments)?;

        // Then process simple variable substitutions
        for (key, value) in arguments {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Remove any remaining unmatched placeholders for optional arguments
        result = self.clean_unmatched_placeholders(&result);

        Ok(result)
    }

    /// Process conditional blocks in the template.
    fn process_conditionals(
        &self,
        template: &str,
        arguments: &HashMap<String, String>,
    ) -> Result<String, PromptError> {
        let mut result = template.to_string();

        // Process {{#if variable}}...{{else}}...{{/if}} blocks
        loop {
            let if_start = result.find("{{#if ");
            if if_start.is_none() {
                break;
            }

            let if_start = if_start.unwrap();
            let var_end = result[if_start..]
                .find("}}")
                .ok_or_else(|| PromptError::template("Unclosed {{#if}} tag"))?;
            let var_end = if_start + var_end;

            let var_name = &result[if_start + 6..var_end];
            let var_name = var_name.trim();

            // Find the matching {{/if}}
            let endif_tag = "{{/if}}";
            let endif_pos = result[var_end..]
                .find(endif_tag)
                .ok_or_else(|| PromptError::template("Missing {{/if}} tag"))?;
            let endif_pos = var_end + endif_pos;

            let block_content = &result[var_end + 2..endif_pos];

            // Check for {{else}}
            let (true_content, false_content) =
                if let Some(else_pos) = block_content.find("{{else}}") {
                    (&block_content[..else_pos], &block_content[else_pos + 8..])
                } else {
                    (block_content, "")
                };

            // Determine if the variable is set and non-empty
            let is_set = arguments
                .get(var_name)
                .map(|v| !v.is_empty())
                .unwrap_or(false);

            let replacement = if is_set { true_content } else { false_content };

            result = format!(
                "{}{}{}",
                &result[..if_start],
                replacement,
                &result[endif_pos + endif_tag.len()..]
            );
        }

        Ok(result)
    }

    /// Remove any unmatched placeholder variables.
    fn clean_unmatched_placeholders(&self, template: &str) -> String {
        let mut result = template.to_string();
        let mut start = 0;

        while let Some(pos) = result[start..].find("{{") {
            let abs_pos = start + pos;
            if let Some(end_pos) = result[abs_pos..].find("}}") {
                let end_abs = abs_pos + end_pos + 2;
                let placeholder = &result[abs_pos..end_abs];

                // Only remove simple placeholders, not special tags
                if !placeholder.contains('#') && !placeholder.contains('/') {
                    result = format!("{}{}", &result[..abs_pos], &result[end_abs..]);
                    // Don't advance start, as we've removed content
                    continue;
                }
            }
            start = abs_pos + 2;
        }

        result
    }
}

/// Builder for creating prompt templates.
pub struct PromptTemplateBuilder {
    name: String,
    description: Option<String>,
    arguments: Vec<PromptArgument>,
    template: String,
}

impl PromptTemplateBuilder {
    /// Create a new builder with the required name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            arguments: Vec::new(),
            template: String::new(),
        }
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a required argument.
    pub fn required_arg(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.arguments.push(PromptArgument {
            name: name.into(),
            title: None,
            description: Some(description.into()),
            required: Some(true),
        });
        self
    }

    /// Add an optional argument.
    pub fn optional_arg(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.arguments.push(PromptArgument {
            name: name.into(),
            title: None,
            description: Some(description.into()),
            required: Some(false),
        });
        self
    }

    /// Set the template string.
    pub fn template(mut self, template: impl Into<String>) -> Self {
        self.template = template.into();
        self
    }

    /// Build the prompt template.
    pub fn build(self) -> PromptTemplate {
        PromptTemplate {
            name: self.name,
            description: self.description,
            arguments: self.arguments,
            template: self.template,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_substitution() {
        let template = PromptTemplate::new("test", None, vec![], "Hello, {{name}}!");

        let mut args = HashMap::new();
        args.insert("name".to_string(), "World".to_string());

        let result = template.render(&args).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_conditional_with_value() {
        let template =
            PromptTemplate::new("test", None, vec![], "Hello{{#if name}}, {{name}}{{/if}}!");

        let mut args = HashMap::new();
        args.insert("name".to_string(), "World".to_string());

        let result = template.render(&args).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_conditional_without_value() {
        let template =
            PromptTemplate::new("test", None, vec![], "Hello{{#if name}}, {{name}}{{/if}}!");

        let args = HashMap::new();

        let result = template.render(&args).unwrap();
        assert_eq!(result, "Hello!");
    }

    #[test]
    fn test_conditional_with_else() {
        let template = PromptTemplate::new(
            "test",
            None,
            vec![],
            "Hello, {{#if name}}{{name}}{{else}}stranger{{/if}}!",
        );

        let args = HashMap::new();

        let result = template.render(&args).unwrap();
        assert_eq!(result, "Hello, stranger!");
    }

    #[test]
    fn test_builder() {
        let template = PromptTemplateBuilder::new("greeting")
            .description("A greeting prompt")
            .required_arg("name", "The name to greet")
            .optional_arg("style", "The greeting style")
            .template("Hello, {{name}}!")
            .build();

        assert_eq!(template.name, "greeting");
        assert_eq!(template.arguments.len(), 2);
    }
}
