//! MCP Server Library
//!
//! This crate provides a scalable Model Context Protocol (MCP) server template
//! with a modular architecture organized by domains.
//!
//! # Architecture
//!
//! The server is organized into the following modules:
//!
//! - **core**: Core infrastructure including configuration, error handling, and the main server
//! - **domains**: Business logic organized by bounded contexts
//!   - **tools**: MCP tools that can be executed by clients
//!   - **resources**: Data resources that can be read by clients
//!   - **prompts**: Prompt templates for consistent interactions
//!
//! # Example
//!
//! ```rust,no_run
//! use music_mcp_server::{core::McpServer, core::Config};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::from_env();
//!     let server = McpServer::new(config);
//!     // Start the server...
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod domains;

// Re-export commonly used types for convenience
pub use core::{Config, Error, McpServer, Result};
