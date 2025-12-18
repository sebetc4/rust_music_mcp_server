//! MCP Server implementation and lifecycle management.
//!
//! This module contains the main server handler that implements the MCP
//! protocol by delegating to domain-specific services.
//!
//! ## Tool Architecture
//!
//! Tools are defined in `domains/tools/definitions/` with one file per tool.
//! Each tool defines:
//! - Parameters struct (for rmcp)
//! - `execute()` method (core logic)
//! - `http_handler()` method (called via ToolRegistry for HTTP transport)
//!
//! The ToolRouter is built dynamically in `domains/tools/router.rs`.
//! **Adding a new tool does NOT require modifying this file!**

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, handler::server::tool::ToolRouter, model::*,
    service::RequestContext, tool_handler,
};
use std::sync::Arc;
use tracing::{info, instrument};

use super::config::Config;
use crate::domains::{
    prompts::PromptService, resources::ResourceService, tools::build_tool_router,
};

#[cfg(feature = "http")]
use crate::domains::tools::ToolRegistry;

/// The main MCP server handler.
///
/// This struct implements the `ServerHandler` trait from rmcp and coordinates
/// between different domain services to handle MCP protocol messages.
#[derive(Clone)]
pub struct McpServer {
    /// Server configuration.
    config: Arc<Config>,

    /// Service for handling resource-related requests.
    resource_service: Arc<ResourceService>,

    /// Service for handling prompt-related requests.
    prompt_service: Arc<PromptService>,

    /// Tool router for handling tool calls.
    tool_router: ToolRouter<Self>,
}

impl McpServer {
    /// Create a new MCP server with the given configuration.
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);

        let resource_service = Arc::new(ResourceService::new(config.resources.clone()));
        let prompt_service = Arc::new(PromptService::new(config.prompts.clone()));

        Self {
            tool_router: build_tool_router::<Self>(config.clone()),
            config,
            resource_service,
            prompt_service,
        }
    }

    /// Get the server name.
    pub fn name(&self) -> &str {
        &self.config.server.name
    }

    /// Get the server version.
    pub fn version(&self) -> &str {
        &self.config.server.version
    }

    /// Get the server configuration (for tool access).
    pub fn config(&self) -> &Arc<Config> {
        &self.config
    }

    // ========================================================================
    // HTTP Transport Support Methods
    // ========================================================================

    /// List all available tools (for HTTP transport).
    pub fn list_tools(&self) -> Vec<serde_json::Value> {
        self.tool_router
            .list_all()
            .into_iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            })
            .collect()
    }

    /// Call a tool by name (for HTTP transport).
    ///
    /// This method uses the ToolRegistry to dispatch to the appropriate
    /// tool handler. Each tool's http_handler is defined in its own file
    /// under `domains/tools/definitions/`.
    #[cfg(feature = "http")]
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let registry = ToolRegistry::new(self.config.clone());
        registry.call_tool(name, arguments)
    }

    /// List all available resources (for HTTP transport).
    pub async fn list_resources(&self) -> Vec<serde_json::Value> {
        let resources = self.resource_service.list_resources().await;

        resources
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "uri": r.uri,
                    "name": r.name,
                    "description": r.description,
                    "mimeType": r.mime_type
                })
            })
            .collect()
    }

    /// Read a resource by URI (for HTTP transport).
    pub async fn read_resource(&self, uri: &str) -> Result<serde_json::Value, String> {
        match self.resource_service.read_resource(uri).await {
            Ok(result) => Ok(serde_json::json!({
                "contents": result.contents
            })),
            Err(e) => Err(e.to_string()),
        }
    }

    /// List all available resource templates (for HTTP transport).
    pub async fn list_resource_templates(&self) -> Vec<serde_json::Value> {
        let templates = self.resource_service.list_resource_templates().await;

        templates
            .into_iter()
            .map(|t| {
                serde_json::json!({
                    "uriTemplate": t.raw.uri_template,
                    "name": t.raw.name,
                    "title": t.raw.title,
                    "description": t.raw.description,
                    "mimeType": t.raw.mime_type
                })
            })
            .collect()
    }

    /// List all available prompts (for HTTP transport).
    pub async fn list_prompts(&self) -> Vec<serde_json::Value> {
        let prompts = self.prompt_service.list_prompts().await;

        prompts
            .into_iter()
            .map(|p| {
                serde_json::json!({
                    "name": p.name,
                    "description": p.description,
                    "arguments": p.arguments
                })
            })
            .collect()
    }

    /// Get a prompt by name (for HTTP transport).
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        // Convert serde_json::Value to HashMap<String, String>
        let args = arguments.and_then(|v| {
            v.as_object().map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
        });

        match self.prompt_service.get_prompt(name, args).await {
            Ok(result) => Ok(serde_json::json!({
                "description": result.description,
                "messages": result.messages
            })),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// ServerHandler implementation with tool_handler macro for automatic tool routing.
#[tool_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "This is a template MCP server. It provides example tools, resources, and prompts."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
            ..Default::default()
        }
    }

    #[instrument(skip(self, _context))]
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        info!("Listing resources");
        let resources = self.resource_service.list_resources().await;
        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
            meta: None,
        })
    }

    #[instrument(skip(self, _context))]
    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        info!("Listing resource templates");
        let templates = self.resource_service.list_resource_templates().await;
        Ok(ListResourceTemplatesResult {
            resource_templates: templates,
            next_cursor: None,
            meta: None,
        })
    }

    #[instrument(skip(self, _context))]
    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        info!("Reading resource: {}", request.uri);
        self.resource_service
            .read_resource(&request.uri)
            .await
            .map_err(|e| McpError::resource_not_found(e.to_string(), None))
    }

    #[instrument(skip(self, _context))]
    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        info!("Listing prompts");
        let prompts = self.prompt_service.list_prompts().await;
        Ok(ListPromptsResult {
            prompts,
            next_cursor: None,
            meta: None,
        })
    }

    #[instrument(skip(self, _context))]
    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        info!("Getting prompt: {}", request.name);
        // Convert serde_json::Map to HashMap<String, String>
        let arguments = request.arguments.map(|map| {
            map.into_iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                .collect()
        });
        self.prompt_service
            .get_prompt(&request.name, arguments)
            .await
            .map_err(|e| McpError::invalid_params(e.to_string(), None))
    }
}
