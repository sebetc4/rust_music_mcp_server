//! HTTP transport implementation.
//!
//! HTTP server with JSON-RPC over POST requests.
//! This allows standard HTTP clients (curl, browsers, etc.) to communicate with the MCP server.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, instrument, warn};

use super::{TransportConfig, TransportError, TransportResult, config::HttpConfig};
use crate::core::McpServer;

/// HTTP transport handler.
pub struct HttpTransport {
    config: HttpConfig,
}

/// JSON-RPC request structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC response structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    /// Create a success response.
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: Option<serde_json::Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    /// Method not found error.
    pub fn method_not_found(id: Option<serde_json::Value>) -> Self {
        Self::error(id, -32601, "Method not found")
    }

    /// Invalid request error.
    pub fn invalid_request(id: Option<serde_json::Value>) -> Self {
        Self::error(id, -32600, "Invalid Request")
    }

    /// Invalid params error.
    pub fn invalid_params(id: Option<serde_json::Value>, msg: impl Into<String>) -> Self {
        Self::error(id, -32602, msg)
    }

    /// Internal error.
    pub fn internal_error(id: Option<serde_json::Value>, msg: impl Into<String>) -> Self {
        Self::error(id, -32603, msg)
    }
}

/// Application state shared across HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    /// The MCP server instance.
    server: McpServer,
    /// Session state for maintaining conversation context.
    session: Arc<RwLock<Option<SessionState>>>,
}

/// Session state for a client.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SessionState {
    initialized: bool,
    protocol_version: String,
}

impl HttpTransport {
    /// Create a new HTTP transport with the given config.
    pub fn new(config: HttpConfig) -> Self {
        Self { config }
    }

    /// Create from TransportConfig (extracts HTTP config).
    pub fn from_transport_config(config: &TransportConfig) -> Option<Self> {
        match config {
            TransportConfig::Http(http_config) => Some(Self::new(http_config.clone())),
            _ => None,
        }
    }

    /// Get the bind address.
    pub fn address(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
    }

    /// Run the HTTP transport.
    pub async fn run(self, server: McpServer) -> TransportResult<()> {
        let addr = self.address();

        let state = AppState {
            server,
            session: Arc::new(RwLock::new(None)),
        };

        // Build router
        let mut app = Router::new()
            .route(&self.config.rpc_path, post(handle_rpc))
            .route("/health", get(health_check))
            .route("/", get(root_handler))
            .with_state(state);

        // Add CORS if enabled
        if self.config.enable_cors {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);
            app = app.layer(cors);
        }

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| TransportError::bind(&addr, e))?;

        let cors_status = if self.config.enable_cors {
            "enabled"
        } else {
            "disabled"
        };
        info!(
            "Ready - listening on {} (JSON-RPC over HTTP, CORS {})",
            addr, cors_status
        );
        info!("  → JSON-RPC: POST {}", self.config.rpc_path);
        info!("  → Health:   GET /health");

        axum::serve(listener, app)
            .await
            .map_err(|e| TransportError::http(e.to_string()))?;

        Ok(())
    }
}

/// Root handler - provides API info.
async fn root_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "MCP Server",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": "HTTP",
        "endpoints": {
            "rpc": "/mcp",
            "health": "/health"
        },
        "protocol": "JSON-RPC 2.0",
        "documentation": "Send POST requests to /mcp with JSON-RPC messages"
    }))
}

/// Health check endpoint.
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Handle JSON-RPC requests.
#[instrument(skip_all, fields(method))]
async fn handle_rpc(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    tracing::Span::current().record("method", &request.method);
    info!("Received JSON-RPC request: {}", request.method);

    let response = process_request(&state, request).await;

    (StatusCode::OK, Json(response))
}

/// Process a JSON-RPC request and return the response.
async fn process_request(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    // Validate JSON-RPC version
    if request.jsonrpc != "2.0" {
        return JsonRpcResponse::invalid_request(request.id);
    }

    match request.method.as_str() {
        // Initialize the MCP session
        "initialize" => handle_initialize(state, request).await,

        // List available tools
        "tools/list" => handle_tools_list(state, request).await,

        // Call a tool
        "tools/call" => handle_tools_call(state, request).await,

        // List available resources
        "resources/list" => handle_resources_list(state, request).await,

        // List resource templates
        "resources/templates/list" => handle_resources_templates_list(state, request).await,

        // Read a resource
        "resources/read" => handle_resources_read(state, request).await,

        // List available prompts
        "prompts/list" => handle_prompts_list(state, request).await,

        // Get a prompt
        "prompts/get" => handle_prompts_get(state, request).await,

        // Notifications (no response needed for stateless HTTP)
        method if method.starts_with("notifications/") => {
            handle_notification(state, &request).await;
            // Return empty success for notifications
            JsonRpcResponse::success(request.id, serde_json::json!(null))
        }

        // Unknown method
        _ => {
            warn!("Unknown method: {}", request.method);
            JsonRpcResponse::method_not_found(request.id)
        }
    }
}

/// Handle initialize request.
async fn handle_initialize(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("Processing initialize request");

    // Extract client info from params
    let _params = request.params.clone().unwrap_or(serde_json::json!({}));

    // Store session state
    let mut session = state.session.write().await;
    *session = Some(SessionState {
        initialized: true,
        protocol_version: "2024-11-05".to_string(),
    });

    // Return server capabilities
    let result = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        },
        "serverInfo": {
            "name": state.server.name(),
            "version": state.server.version()
        },
        "instructions": "This is a template MCP server. It provides example tools, resources, and prompts."
    });

    JsonRpcResponse::success(request.id, result)
}

/// Handle tools/list request.
async fn handle_tools_list(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("Processing tools/list request");

    let tools = state.server.list_tools();
    let result = serde_json::json!({
        "tools": tools
    });

    JsonRpcResponse::success(request.id, result)
}

/// Handle tools/call request.
async fn handle_tools_call(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("Processing tools/call request");

    let params = match request.params {
        Some(p) => p,
        None => return JsonRpcResponse::invalid_params(request.id.clone(), "Missing params"),
    };

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return JsonRpcResponse::invalid_params(request.id.clone(), "Missing tool name"),
    };

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    match state.server.call_tool(&name, arguments).await {
        Ok(result) => JsonRpcResponse::success(request.id, result),
        Err(e) => JsonRpcResponse::invalid_params(request.id, e.to_string()),
    }
}

/// Handle resources/list request.
async fn handle_resources_list(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("Processing resources/list request");

    let resources = state.server.list_resources().await;
    let result = serde_json::json!({
        "resources": resources
    });

    JsonRpcResponse::success(request.id, result)
}

/// Handle resources/templates/list request.
async fn handle_resources_templates_list(
    state: &AppState,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    info!("Processing resources/templates/list request");

    let templates = state.server.list_resource_templates().await;
    let result = serde_json::json!({
        "resourceTemplates": templates
    });

    JsonRpcResponse::success(request.id, result)
}

/// Handle resources/read request.
async fn handle_resources_read(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("Processing resources/read request");

    let params = match request.params {
        Some(p) => p,
        None => return JsonRpcResponse::invalid_params(request.id.clone(), "Missing params"),
    };

    let uri = match params.get("uri").and_then(|v| v.as_str()) {
        Some(u) => u.to_string(),
        None => return JsonRpcResponse::invalid_params(request.id.clone(), "Missing resource URI"),
    };

    match state.server.read_resource(&uri).await {
        Ok(result) => JsonRpcResponse::success(request.id, result),
        Err(e) => JsonRpcResponse::invalid_params(request.id, e.to_string()),
    }
}

/// Handle prompts/list request.
async fn handle_prompts_list(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("Processing prompts/list request");

    let prompts = state.server.list_prompts().await;
    let result = serde_json::json!({
        "prompts": prompts
    });

    JsonRpcResponse::success(request.id, result)
}

/// Handle prompts/get request.
async fn handle_prompts_get(state: &AppState, request: JsonRpcRequest) -> JsonRpcResponse {
    info!("Processing prompts/get request");

    let params = match request.params {
        Some(p) => p,
        None => return JsonRpcResponse::invalid_params(request.id.clone(), "Missing params"),
    };

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return JsonRpcResponse::invalid_params(request.id.clone(), "Missing prompt name"),
    };

    let arguments = params.get("arguments").cloned();

    match state.server.get_prompt(&name, arguments).await {
        Ok(result) => JsonRpcResponse::success(request.id, result),
        Err(e) => JsonRpcResponse::invalid_params(request.id, e.to_string()),
    }
}

/// Handle notifications (no response needed).
async fn handle_notification(state: &AppState, request: &JsonRpcRequest) {
    match request.method.as_str() {
        "notifications/initialized" => {
            info!("Client sent initialized notification");
            let mut session = state.session.write().await;
            if let Some(ref mut s) = *session {
                s.initialized = true;
            }
        }
        _ => {
            info!("Received notification: {}", request.method);
        }
    }
}
