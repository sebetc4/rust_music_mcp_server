//! Configuration management for the MCP server.
//!
//! This module provides a centralized configuration structure that can be
//! populated from environment variables, configuration files, or defaults.

use super::transport::TransportConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// Main configuration structure for the MCP server.
///
/// This struct contains all configurable aspects of the server, organized
/// by domain for clarity and maintainability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server identification and metadata.
    pub server: ServerConfig,

    /// Resources domain configuration.
    pub resources: ResourcesConfig,

    /// Prompts domain configuration.
    pub prompts: PromptsConfig,

    /// Logging configuration.
    pub logging: LoggingConfig,

    /// Transport configuration.
    pub transport: TransportConfig,

    /// External API credentials configuration.
    pub credentials: CredentialsConfig,

    /// Security and path validation configuration.
    pub security: SecurityConfig,
}

/// Server identification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// The name of the server as reported to clients.
    pub name: String,

    /// The version of the server.
    pub version: String,
}

/// Configuration for the resources domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourcesConfig {
    /// Base directory for file resources (if applicable).
    pub base_path: Option<String>,
    // Resources are registered in domains/resources/registry.rs
}

/// Configuration for the prompts domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsConfig {
    // Prompts are registered in domains/prompts/registry.rs
    // Add prompt-specific configuration here if needed.
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level filter (e.g., "info", "debug", "trace").
    pub level: String,

    /// Whether to include timestamps in log output.
    pub with_timestamps: bool,
}

/// Configuration for external API credentials.
#[derive(Clone, Serialize, Deserialize)]
pub struct CredentialsConfig {
    /// AcoustID API key for audio fingerprinting.
    /// Get a free key at: https://acoustid.org/api-key
    pub acoustid_api_key: Option<String>,
}

/// Custom Debug implementation to redact secrets from logs.
impl std::fmt::Debug for CredentialsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialsConfig")
            .field(
                "acoustid_api_key",
                &self.acoustid_api_key.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

/// Configuration for security and path validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Optional root directory for path operations.
    /// If None, no path restrictions are enforced.
    /// All file system operations will be validated against this root.
    pub root_path: Option<PathBuf>,

    /// Whether to allow symlinks in path validation.
    /// If true, symlinks are followed and their targets are validated.
    /// If false, symlinks pointing outside the root are rejected.
    pub allow_symlinks: bool,
}

impl Default for CredentialsConfig {
    fn default() -> Self {
        Self {
            // Default public key for testing/demo purposes
            acoustid_api_key: Some("Kok2GHQlrAg".to_string()),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            // No root path restriction by default (backwards compatible)
            root_path: None,
            // Allow symlinks by default with validation
            allow_symlinks: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                name: "mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            resources: ResourcesConfig::default(),
            prompts: PromptsConfig::default(),
            logging: LoggingConfig {
                level: "info".to_string(),
                with_timestamps: true,
            },
            transport: TransportConfig::default(),
            credentials: CredentialsConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Config {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from environment variables.
    ///
    /// Environment variables are expected to be prefixed with `MCP_`.
    /// For example: `MCP_SERVER_NAME`, `MCP_LOGGING_LEVEL`.
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let mut config = Self::default();

        if let Ok(name) = std::env::var("MCP_SERVER_NAME") {
            config.server.name = name;
        }

        if let Ok(level) = std::env::var("MCP_LOG_LEVEL") {
            config.logging.level = level;
        }

        if let Ok(base_path) = std::env::var("MCP_RESOURCES_BASE_PATH") {
            config.resources.base_path = Some(base_path);
        }

        // Load transport configuration from environment
        config.transport = TransportConfig::from_env();

        // Load AcoustID API key
        if let Ok(api_key) = std::env::var("MCP_ACOUSTID_API_KEY") {
            config.credentials.acoustid_api_key = Some(api_key);
            info!("AcoustID API key loaded from environment");
        } else {
            warn!(
                "Using default AcoustID API key. For higher rate limits, \
                 set MCP_ACOUSTID_API_KEY (get your key at https://acoustid.org/api-key)"
            );
        }

        // Load security configuration
        if let Ok(root_path) = std::env::var("MCP_ROOT_PATH") {
            config.security.root_path = Some(PathBuf::from(root_path));
            info!("Path security enabled: root directory set to {:?}", config.security.root_path);
        } else {
            warn!(
                "MCP_ROOT_PATH not set - no path restrictions active. \
                 All filesystem paths will be allowed."
            );
        }

        if let Ok(allow_symlinks) = std::env::var("MCP_ALLOW_SYMLINKS") {
            config.security.allow_symlinks = allow_symlinks.parse().unwrap_or(true);
            info!("Symlinks allowed: {}", config.security.allow_symlinks);
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to ensure env var tests run serially
    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_credentials_from_env() {
        let _lock = ENV_TEST_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("MCP_ACOUSTID_API_KEY", "test_key_12345");
        }
        let config = Config::from_env();
        assert_eq!(
            config.credentials.acoustid_api_key.as_deref(),
            Some("test_key_12345")
        );
        unsafe {
            std::env::remove_var("MCP_ACOUSTID_API_KEY");
        }
    }

    #[test]
    fn test_credentials_default_fallback() {
        let _lock = ENV_TEST_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("MCP_ACOUSTID_API_KEY");
        }
        let config = Config::from_env();
        assert_eq!(
            config.credentials.acoustid_api_key.as_deref(),
            Some("Kok2GHQlrAg")
        );
    }

    #[test]
    fn test_credentials_redacted_in_debug() {
        let creds = CredentialsConfig {
            acoustid_api_key: Some("super_secret_key".to_string()),
        };
        let debug_str = format!("{:?}", creds);
        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains("super_secret_key"));
    }

    #[test]
    fn test_config_default_has_credentials() {
        let config = Config::default();
        assert!(config.credentials.acoustid_api_key.is_some());
    }
}
