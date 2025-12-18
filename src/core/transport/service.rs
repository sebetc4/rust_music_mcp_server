//! Transport service - orchestrates different transport types.
//!
//! This service provides a unified interface for starting the MCP server
//! with different transport mechanisms.

use tracing::info;

use super::{TransportConfig, TransportResult};
use crate::core::McpServer;

#[cfg(feature = "stdio")]
use super::stdio::StdioTransport;

#[cfg(feature = "tcp")]
use super::tcp::TcpTransport;

#[cfg(feature = "http")]
use super::http::HttpTransport;

/// Transport service - manages the transport layer for the MCP server.
pub struct TransportService {
    config: TransportConfig,
}

impl TransportService {
    /// Create a new transport service with the given configuration.
    pub fn new(config: TransportConfig) -> Self {
        Self { config }
    }

    /// Create a transport service from environment variables.
    pub fn from_env() -> Self {
        Self::new(TransportConfig::from_env())
    }

    /// Get the transport configuration.
    pub fn config(&self) -> &TransportConfig {
        &self.config
    }

    /// Log information about the configured transport.
    pub fn log_info(&self) {
        info!("Starting transport: {}", self.config.description());
    }

    /// Start the transport with the given MCP server.
    ///
    /// This method blocks until the transport is shut down.
    pub async fn run(self, server: McpServer) -> TransportResult<()> {
        self.log_info();

        match self.config {
            #[cfg(feature = "stdio")]
            TransportConfig::Stdio => StdioTransport::run(server).await,
            #[cfg(feature = "tcp")]
            TransportConfig::Tcp(cfg) => TcpTransport::new(cfg).run(server).await,
            #[cfg(feature = "http")]
            TransportConfig::Http(cfg) => HttpTransport::new(cfg).run(server).await,
        }
    }
}

/// Builder for creating a transport service with custom options.
#[allow(dead_code)]
pub struct TransportServiceBuilder {
    config: TransportConfig,
}

#[allow(dead_code)]
impl TransportServiceBuilder {
    /// Create a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: TransportConfig::default(),
        }
    }

    /// Use STDIO transport.
    #[cfg(feature = "stdio")]
    pub fn stdio(mut self) -> Self {
        self.config = TransportConfig::Stdio;
        self
    }

    /// Use TCP transport on the given port.
    #[cfg(feature = "tcp")]
    pub fn tcp(mut self, port: u16, host: impl Into<String>) -> Self {
        self.config = TransportConfig::tcp(port, host);
        self
    }

    /// Use HTTP transport on the given port.
    #[cfg(feature = "http")]
    pub fn http(mut self, port: u16, host: impl Into<String>) -> Self {
        self.config = TransportConfig::http(port, host);
        self
    }

    /// Use configuration from environment variables.
    pub fn from_env(mut self) -> Self {
        self.config = TransportConfig::from_env();
        self
    }

    /// Build the transport service.
    pub fn build(self) -> TransportService {
        TransportService::new(self.config)
    }
}

impl Default for TransportServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
