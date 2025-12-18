//! Prompt service implementation.
//!
//! The PromptService manages prompt templates and their instantiation.
//! It maintains a registry of available prompts and handles argument substitution.
//!
//! Prompts are defined in `definitions/` and registered via `registry.rs`.
//! Adding a new prompt does NOT require modifying this file.

use rmcp::model::{GetPromptResult, Prompt, PromptMessage, PromptMessageRole};
use std::collections::HashMap;
use tracing::info;

use super::error::PromptError;
use super::registry::get_all_prompts;
use super::templates::PromptTemplate;
use crate::core::config::PromptsConfig;

/// Service for managing and instantiating prompts.
///
/// This service maintains a registry of prompt templates and handles
/// prompt listing and argument substitution.
pub struct PromptService {
    /// Configuration for the prompts domain.
    #[allow(dead_code)]
    config: PromptsConfig,

    /// Registry of available prompts.
    /// Key: prompt name, Value: prompt template
    prompts: HashMap<String, PromptTemplate>,
}

impl PromptService {
    /// Create a new PromptService with the given configuration.
    pub fn new(config: PromptsConfig) -> Self {
        info!("Initializing PromptService");

        let mut service = Self {
            config,
            prompts: HashMap::new(),
        };

        // Register all prompts from registry
        service.register_from_registry();

        service
    }

    /// Register all prompts from the registry.
    fn register_from_registry(&mut self) {
        info!("Registering prompts from registry");
        for template in get_all_prompts() {
            self.register_prompt(template);
        }
    }

    /// Register a prompt template.
    pub fn register_prompt(&mut self, template: PromptTemplate) {
        info!("Registering prompt: {}", template.name);
        self.prompts.insert(template.name.clone(), template);
    }

    /// List all available prompts.
    pub async fn list_prompts(&self) -> Vec<Prompt> {
        self.prompts
            .values()
            .map(|template| Prompt {
                name: template.name.clone(),
                title: None,
                description: template.description.clone(),
                arguments: Some(template.arguments.clone()),
                icons: None,
                meta: None,
            })
            .collect()
    }

    /// Get a prompt with arguments substituted.
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> Result<GetPromptResult, PromptError> {
        let template = self
            .prompts
            .get(name)
            .ok_or_else(|| PromptError::not_found(name))?;

        let arguments = arguments.unwrap_or_default();

        // Validate required arguments
        for arg in &template.arguments {
            if arg.required.unwrap_or(false) && !arguments.contains_key(&arg.name) {
                return Err(PromptError::missing_argument(&arg.name));
            }
        }

        // Render the template
        let content = template.render(&arguments)?;

        Ok(GetPromptResult {
            description: template.description.clone(),
            messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prompt_service_creation() {
        let config = PromptsConfig::default();
        let service = PromptService::new(config);

        let prompts = service.list_prompts().await;
        assert!(!prompts.is_empty());
    }

    #[tokio::test]
    async fn test_get_prompt_with_arguments() {
        let config = PromptsConfig::default();
        let service = PromptService::new(config);

        let mut args = HashMap::new();
        args.insert("name".to_string(), "World".to_string());

        let result = service.get_prompt("greeting", Some(args)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_prompt_missing_required_argument() {
        let config = PromptsConfig::default();
        let service = PromptService::new(config);

        let result = service.get_prompt("greeting", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_nonexistent_prompt() {
        let config = PromptsConfig::default();
        let service = PromptService::new(config);

        let result = service.get_prompt("nonexistent", None).await;
        assert!(result.is_err());
    }
}
