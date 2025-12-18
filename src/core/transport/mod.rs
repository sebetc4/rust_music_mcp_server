//! Transport layer for the MCP server.
//!
//! This module provides different transport implementations:
//! - **STDIO**: Standard input/output (default for MCP) - feature: `stdio`
//! - **TCP**: Raw TCP socket with JSON-RPC messages - feature: `tcp`
//! - **HTTP**: HTTP server with JSON-RPC over POST requests - feature: `http`
//!
//! Each transport handles the connection lifecycle and delegates
//! message processing to the MCP server handler.
//!
//! # Feature Flags
//!
//! Transport implementations are conditionally compiled based on features:
//! - `stdio` (default): STDIO transport - minimal dependencies
//! - `tcp`: TCP transport - adds tokio/net
//! - `http`: HTTP transport - adds axum, tower, tower-http

mod config;
mod error;
mod service;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "tcp")]
pub mod tcp;

#[cfg(feature = "stdio")]
pub mod stdio;

pub use config::TransportConfig;
pub use error::{TransportError, TransportResult};
pub use service::TransportService;

// Re-export configs for convenience
#[cfg(feature = "tcp")]
pub use config::TcpConfig;

#[cfg(feature = "http")]
pub use config::HttpConfig;
