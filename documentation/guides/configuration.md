# Configuration Guide

This guide explains how to configure the MCP server using environment variables.

## Table of Contents

- [Overview](#overview)
- [Environment Variables](#environment-variables)
- [Configuration Workflow](#configuration-workflow)
- [Security Best Practices](#security-best-practices)
- [Examples](#examples)

## Overview

The MCP server uses a centralized configuration system based on environment variables. All configuration is loaded at **runtime** (when the server starts), not at compile time.

### Configuration Architecture

```
.env file (optional)
    ↓
Environment Variables
    ↓
Config::from_env()
    ↓
Config struct (Arc-wrapped)
    ↓
Distributed to services
```

### Key Features

- ✅ **Runtime Configuration**: All settings loaded from environment at startup
- ✅ **Type-Safe**: Strong typing with validation
- ✅ **Secure**: Sensitive data (API keys) redacted from logs
- ✅ **Flexible**: Support for `.env` files and system environment variables
- ✅ **Documented**: All variables have defaults and clear documentation

## Environment Variables

All environment variables use the `MCP_` prefix to avoid conflicts.

### Server Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `MCP_SERVER_NAME` | String | `"mcp-server"` | Server name reported to clients |
| `MCP_LOG_LEVEL` | String | `"info"` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `MCP_RESOURCES_BASE_PATH` | String | None | Optional base path for file resources |

### Transport Configuration

#### STDIO Transport (Default)

No configuration needed. STDIO is the default MCP transport.

```bash
# Use default STDIO transport
# No MCP_TRANSPORT variable needed
```

#### TCP Transport

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `MCP_TRANSPORT` | String | `"stdio"` | Set to `"tcp"` to enable TCP transport |
| `MCP_TCP_PORT` | u16 | `3000` | TCP port to listen on |
| `MCP_TCP_HOST` | String | `"127.0.0.1"` | TCP host address to bind |

```bash
MCP_TRANSPORT=tcp
MCP_TCP_PORT=3000
MCP_TCP_HOST=127.0.0.1
```

#### HTTP Transport

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `MCP_TRANSPORT` | String | `"stdio"` | Set to `"http"` to enable HTTP transport |
| `MCP_HTTP_PORT` | u16 | `8080` | HTTP port to listen on |
| `MCP_HTTP_HOST` | String | `"127.0.0.1"` | HTTP host address to bind |
| `MCP_HTTP_PATH` | String | `"/mcp"` | RPC endpoint path |
| `MCP_HTTP_CORS` | Boolean | `true` | Enable CORS for browser clients |

```bash
MCP_TRANSPORT=http
MCP_HTTP_PORT=8080
MCP_HTTP_HOST=127.0.0.1
MCP_HTTP_PATH=/mcp
MCP_HTTP_CORS=true
```

### External API Credentials

#### AcoustID API (Audio Fingerprinting)

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `MCP_ACOUSTID_API_KEY` | String | Public demo key | AcoustID API key for audio identification |

**⚠️ Important**: The default key has rate limits. For production use, get your own free key at https://acoustid.org/api-key

```bash
# Your personal API key
MCP_ACOUSTID_API_KEY=your_api_key_here
```

**Benefits of custom API key**:
- Higher rate limits
- Better performance
- No shared quota with other users

### Security Configuration

#### Path Security

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `MCP_ROOT_PATH` | Path | None | Root directory for all file operations. If set, restricts access to this directory and its subdirectories |
| `MCP_ALLOW_SYMLINKS` | Boolean | `true` | Whether to follow symlinks. If `true`, symlinks are followed and validated; if `false`, symlinks pointing outside root are rejected |

**Path Security Overview**:

The path security system validates all file/directory paths used by filesystem tools (`fs_list_dir`, `fs_rename`, `read_metadata`, `write_metadata`, `mb_identify_record`) to ensure they stay within configured boundaries.

**Key Features**:
- ✅ **Path Traversal Protection**: Blocks `../../../etc/passwd` attacks
- ✅ **Symlink Validation**: Validates symlink destinations are within bounds
- ✅ **Relative Path Resolution**: Safely resolves relative paths like `./music`
- ✅ **Canonical Path Validation**: Uses OS path canonicalization for security
- ✅ **Clear Error Messages**: Users get explicit feedback when paths are rejected

**Examples**:

```bash
# Development: No restrictions (default)
# MCP_ROOT_PATH is not set - all paths allowed

# Production: Restrict to music library
MCP_ROOT_PATH=/home/user/music
MCP_ALLOW_SYMLINKS=true

# High security: No symlinks allowed
MCP_ROOT_PATH=/var/music
MCP_ALLOW_SYMLINKS=false
```

**Behavior**:

When `MCP_ROOT_PATH` is set:
- ✅ `/home/user/music/albums/song.mp3` → **ALLOWED**
- ✅ `/home/user/music/../music/song.mp3` → **ALLOWED** (resolves to valid path)
- ❌ `/home/user/documents/file.txt` → **BLOCKED**
- ❌ `/home/user/music/../documents/file.txt` → **BLOCKED** (path traversal detected)

When `MCP_ROOT_PATH` is **not** set:
- ✅ All paths allowed (backwards compatible)
- ⚠️ Warning logged at startup

**Recommended Setup**:

| Environment | MCP_ROOT_PATH | MCP_ALLOW_SYMLINKS | Rationale |
|-------------|---------------|---------------------|-----------|
| Development | Not set | N/A | Flexibility for testing |
| Production | Always set | `true` | Security with flexibility |
| High Security | Always set | `false` | Maximum security, no symlinks |
| Docker | `/data` or `/music` | `true` | Container volume mount |

## Configuration Workflow

### 1. Startup Sequence

```rust
main()
  ↓
Config::from_env()                    // Load from environment
  ↓
  ├─ dotenvy::dotenv().ok()          // Load .env file if present
  ├─ std::env::var("MCP_*")          // Read each variable
  └─ Apply defaults for missing vars
  ↓
Arc::new(config)                      // Wrap in Arc for sharing
  ↓
McpServer::new(config)                // Pass to server
  ↓
  ├─ ResourceService(config)          // Services receive config
  ├─ PromptService(config)
  └─ ToolRouter(config)               // Tools can access config
```

### 2. Configuration Sources (Priority Order)

1. **System Environment Variables** (highest priority)
2. **`.env` file** (if present)
3. **Default values** (fallback)

Example:
```bash
# System environment takes precedence
export MCP_LOG_LEVEL=debug

# .env file value is ignored if system env is set
# .env: MCP_LOG_LEVEL=info

# Result: LOG_LEVEL = "debug"
```

### 3. Using `.env` Files

Create a `.env` file in the project root:

```bash
# .env
MCP_TRANSPORT=http
MCP_HTTP_PORT=4000
MCP_ACOUSTID_API_KEY=your_key_here
MCP_LOG_LEVEL=debug
```

**Benefits**:
- ✅ Easy to manage during development
- ✅ Not committed to version control (`.gitignore`)
- ✅ Can be shared across team (use `.env.example`)

**⚠️ Warning**: Never commit `.env` files with real API keys!

### 4. Config Structure in Code

```rust
pub struct Config {
    pub server: ServerConfig,           // Server metadata
    pub resources: ResourcesConfig,     // Resource paths
    pub prompts: PromptsConfig,         // Prompt settings
    pub logging: LoggingConfig,         // Log configuration
    pub transport: TransportConfig,     // Transport (stdio/tcp/http)
    pub credentials: CredentialsConfig, // API keys (SECURE)
}
```

### 5. How Tools Access Configuration

Tools that need configuration receive it via dependency injection:

```rust
// STDIO/TCP Path
ToolRoute::new_dyn(tool, move |ctx| {
    let config = config.clone();  // Captured in closure
    async move {
        Tool::execute(&params, &config)
    }
})

// HTTP Path
ToolRegistry::new(config)
    ↓
registry.call_tool(name, args)
    ↓
Tool::http_handler(args, config)
```

**Example**: `mb_identify_record` tool accesses AcoustID API key:

```rust
pub fn execute(params: &Params, config: &Config) -> Result {
    // Extract API key from config
    let api_key = config.credentials.acoustid_api_key
        .as_deref()
        .unwrap_or_default();

    // Use the key...
}
```

## Security Best Practices

### 1. API Key Security

✅ **DO**:
- Use environment variables for API keys
- Use `.env` files for development
- Use system environment variables for production
- Get your own API keys (don't rely on defaults)
- Add `.env` to `.gitignore`

❌ **DON'T**:
- Hardcode API keys in source code
- Commit `.env` files to version control
- Share API keys in public repositories
- Use default API keys in production

### 2. Log Security

API keys are automatically **redacted from logs**:

```rust
// Custom Debug implementation
impl Debug for CredentialsConfig {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("CredentialsConfig")
            .field("acoustid_api_key", &"[REDACTED]")
            .finish()
    }
}
```

**Example log output**:
```
Config {
    credentials: CredentialsConfig {
        acoustid_api_key: "[REDACTED]"
    }
}
```

### 3. Path Security

**⚠️ CRITICAL for Production**: Always set `MCP_ROOT_PATH` in production to prevent unauthorized file access.

✅ **DO**:
- Set `MCP_ROOT_PATH` to your music library directory
- Use absolute paths for `MCP_ROOT_PATH`
- Validate the root path exists before starting the server
- Use Docker volume mounts aligned with `MCP_ROOT_PATH`
- Log path validation errors for security monitoring
- Test path restrictions before deploying

❌ **DON'T**:
- Leave `MCP_ROOT_PATH` unset in production
- Use `/` (root filesystem) as `MCP_ROOT_PATH`
- Disable path security for convenience
- Ignore path validation warnings
- Give write access outside your music directory

**Example Production Setup**:

```bash
# Secure production configuration
MCP_ROOT_PATH=/var/music/library
MCP_ALLOW_SYMLINKS=true
MCP_LOG_LEVEL=info

# Server will only access files in /var/music/library and subdirectories
# All path traversal attempts will be blocked and logged
```

**Security Threat Mitigation**:

| Threat | Mitigation | Status |
|--------|-----------|--------|
| Path traversal (`../../../etc/passwd`) | Canonical path validation | ✅ Blocked |
| Symlink attacks (link to `/etc`) | Symlink destination validation | ✅ Blocked |
| Absolute path bypass (`/etc/shadow`) | Root directory boundary check | ✅ Blocked |
| Relative path confusion | Path resolution before validation | ✅ Blocked |

### 4. Production Deployment

**Recommended approach**:

```bash
# Set environment variables in your deployment system
# Example: Docker
docker run \
  -e MCP_ACOUSTID_API_KEY=prod_key \
  -e MCP_ROOT_PATH=/music \
  -v /host/music:/music:ro \
  music_mcp_server

# Example: systemd service
[Service]
Environment="MCP_ACOUSTID_API_KEY=prod_key"
Environment="MCP_TRANSPORT=tcp"
Environment="MCP_TCP_PORT=3000"
Environment="MCP_ROOT_PATH=/var/music/library"
Environment="MCP_ALLOW_SYMLINKS=true"
```

## Examples

### Example 1: Development Setup

```bash
# .env
MCP_TRANSPORT=http
MCP_HTTP_PORT=4000
MCP_LOG_LEVEL=debug
MCP_ACOUSTID_API_KEY=dev_key_abc123
```

```bash
# Start server
cargo run --features http
```

### Example 2: Production Deployment

```bash
# Set system environment variables
export MCP_TRANSPORT=tcp
export MCP_TCP_PORT=3000
export MCP_TCP_HOST=0.0.0.0
export MCP_LOG_LEVEL=info
export MCP_ACOUSTID_API_KEY=prod_key_xyz789
export MCP_ROOT_PATH=/var/music/library  # IMPORTANT: Restrict filesystem access
export MCP_ALLOW_SYMLINKS=true

# Build and run
cargo build --release --features tcp
./target/release/music_mcp_server
```

**Expected startup logs**:
```
INFO Path security enabled: root directory set to "/var/music/library"
INFO Symlinks allowed: true
INFO MCP server listening on tcp://0.0.0.0:3000
```

### Example 3: Testing with Custom Config

```rust
#[test]
fn test_with_custom_config() {
    // Create config programmatically
    let mut config = Config::default();
    config.credentials.acoustid_api_key = Some("test_key".to_string());

    // Use in test
    let result = Tool::execute(&params, &config);
    assert!(result.is_ok());
}
```

### Example 4: Docker Deployment

```dockerfile
# Dockerfile
FROM rust:latest
WORKDIR /app
COPY . .
RUN cargo build --release --features tcp

# Use environment variables at runtime
ENV MCP_TRANSPORT=tcp
ENV MCP_TCP_PORT=3000

CMD ["./target/release/music_mcp_server"]
```

```bash
# Run with custom API key
docker run \
  -e MCP_ACOUSTID_API_KEY=your_key \
  -p 3000:3000 \
  music_mcp_server
```

### Example 5: Multiple Environments

```bash
# .env.development
MCP_TRANSPORT=http
MCP_HTTP_PORT=4000
MCP_LOG_LEVEL=debug
MCP_ACOUSTID_API_KEY=dev_key

# .env.production
MCP_TRANSPORT=tcp
MCP_TCP_PORT=3000
MCP_LOG_LEVEL=info
MCP_ACOUSTID_API_KEY=prod_key

# Load appropriate file
cp .env.development .env  # for development
cargo run --features http
```

## Troubleshooting

### Issue: "Using default AcoustID API key" warning

**Cause**: `MCP_ACOUSTID_API_KEY` not set

**Solution**:
```bash
# Get your free API key
# Visit: https://acoustid.org/api-key

# Set in .env
echo "MCP_ACOUSTID_API_KEY=your_key_here" >> .env
```

### Issue: Config not loading from `.env`

**Cause**: `.env` file not in correct location or not readable

**Solution**:
```bash
# Ensure .env is in project root
ls -la .env

# Check file permissions
chmod 644 .env

# Verify content
cat .env
```

### Issue: Environment variable not taking effect

**Cause**: System environment variable overriding `.env`

**Solution**:
```bash
# Check system environment
echo $MCP_ACOUSTID_API_KEY

# Unset if needed
unset MCP_ACOUSTID_API_KEY

# Or override explicitly
MCP_ACOUSTID_API_KEY=new_key cargo run
```

## Adding New Configuration Options

To add a new configuration option:

1. **Add field to appropriate config struct** (`src/core/config.rs`):
```rust
pub struct CredentialsConfig {
    pub acoustid_api_key: Option<String>,
    pub spotify_api_key: Option<String>,  // New field
}
```

2. **Update `from_env()` method**:
```rust
if let Ok(key) = std::env::var("MCP_SPOTIFY_API_KEY") {
    config.credentials.spotify_api_key = Some(key);
}
```

3. **Update default implementation** if needed:
```rust
impl Default for CredentialsConfig {
    fn default() -> Self {
        Self {
            acoustid_api_key: Some("default_key".to_string()),
            spotify_api_key: None,  // No default for Spotify
        }
    }
}
```

4. **Document in this guide**

5. **Add to `.env.example`**:
```bash
# Spotify API credentials
# MCP_SPOTIFY_API_KEY=your_spotify_key
```

## See Also

- [Architecture Overview](../architecture/overview.md)
- [Adding Tools Guide](adding-tools.md)
- [Tool Documentation](../tools/README.md)
- [Config Module Documentation](../../src/core/config.rs)
