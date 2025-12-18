//! STDIO transport implementation.
//!
//! Standard input/output transport for MCP - the default and recommended mode.

use rmcp::ServiceExt;
use tracing::info;

use super::{TransportError, TransportResult};
use crate::core::McpServer;

/// STDIO transport handler.
pub struct StdioTransport;

impl StdioTransport {
    /// Run the STDIO transport.
    pub async fn run(server: McpServer) -> TransportResult<()> {
        info!("Ready - communicating via stdin/stdout");

        let service = server
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| TransportError::init(e.to_string()))?;

        service
            .waiting()
            .await
            .map_err(|e| TransportError::ServiceError(e.to_string()))?;

        info!("STDIO transport finished");
        Ok(())
    }
}
