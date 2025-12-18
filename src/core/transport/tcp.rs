//! TCP transport implementation.
//!
//! Raw TCP socket transport with JSON-RPC messages (line-delimited).

use rmcp::ServiceExt;
use tokio::net::TcpListener;
use tracing::{info, warn};

use super::{TransportConfig, TransportError, TransportResult, config::TcpConfig};
use crate::core::McpServer;

/// TCP transport handler.
pub struct TcpTransport {
    config: TcpConfig,
}

impl TcpTransport {
    /// Create a new TCP transport with the given config.
    pub fn new(config: TcpConfig) -> Self {
        Self { config }
    }

    /// Create from TransportConfig (extracts TCP config).
    pub fn from_transport_config(config: &TransportConfig) -> Option<Self> {
        match config {
            TransportConfig::Tcp(tcp_config) => Some(Self::new(tcp_config.clone())),
            _ => None,
        }
    }

    /// Get the bind address.
    pub fn address(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
    }

    /// Run the TCP transport.
    pub async fn run(self, server: McpServer) -> TransportResult<()> {
        let addr = self.address();

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| TransportError::bind(&addr, e))?;

        info!("Ready - listening on {} (JSON-RPC over TCP)", addr);

        // Accept multiple connections in a loop
        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    info!("Accepted connection from {}", peer_addr);

                    // Set TCP_NODELAY to disable Nagle's algorithm
                    if let Err(e) = stream.set_nodelay(true) {
                        warn!("Failed to set TCP_NODELAY for {}: {}", peer_addr, e);
                    }

                    let server_clone = server.clone();

                    // Spawn a task to handle this connection
                    tokio::spawn(async move {
                        Self::handle_connection(server_clone, stream, peer_addr).await;
                    });
                }
                Err(e) => {
                    warn!("Failed to accept connection: {}", e);
                    // Small delay to avoid spinning on persistent errors
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Handle a single TCP connection.
    async fn handle_connection(
        server: McpServer,
        stream: tokio::net::TcpStream,
        peer_addr: std::net::SocketAddr,
    ) {
        // Initialize the MCP service for this connection
        let service = match server.serve(stream).await {
            Ok(s) => {
                info!("Client {} connected, serving...", peer_addr);
                s
            }
            Err(e) => {
                warn!("Failed to initialize service for {}: {}", peer_addr, e);
                return;
            }
        };

        // Handle requests from this client
        if let Err(e) = service.waiting().await {
            warn!("Error while serving client {}: {}", peer_addr, e);
            warn!("Error details: {:?}", e);
        } else {
            info!("Client {} disconnected cleanly", peer_addr);
        }
    }
}
