# Adding New MCP Tools

This guide walks you through the process of creating a new MCP tool from scratch.

---

## Table of Contents

1. [Tool Anatomy](#tool-anatomy)
2. [Step-by-Step Tutorial](#step-by-step-tutorial)
3. [Complete Example](#complete-example)
4. [Testing Your Tool](#testing-your-tool)
5. [Common Pitfalls](#common-pitfalls)

---

## Tool Anatomy

Every MCP tool in this project follows a consistent structure:

```
Tool Module
├── Parameter struct (with serde + JsonSchema)
├── Tool struct (empty, holds constants and methods)
│   ├── NAME: &str constant
│   ├── DESCRIPTION: &str constant
│   ├── execute() method (core logic)
│   ├── http_handler() method (HTTP transport)
│   ├── to_tool() method (metadata)
│   └── create_route() method (STDIO/TCP route)
├── Helper functions (if needed)
└── Tests module
```

### Key Components

| Component | Purpose | Required |
|-----------|---------|----------|
| **Parameter Struct** | Defines tool inputs with validation | Yes |
| **NAME** | Tool identifier in MCP protocol | Yes |
| **DESCRIPTION** | Tool description for clients | Yes |
| **execute()** | Core business logic | Yes |
| **http_handler()** | HTTP-specific handler | Only if HTTP support needed |
| **to_tool()** | Tool metadata for MCP | Yes |
| **create_route()** | STDIO/TCP route registration | Yes |

---

## Step-by-Step Tutorial

### Step 1: Create Tool File

Create a new file in the appropriate subdirectory:

- **Filesystem tools**: `src/domains/tools/definitions/fs/your_tool.rs`
- **Metadata tools**: `src/domains/tools/definitions/metadata/your_tool.rs`
- **MusicBrainz tools**: `src/domains/tools/definitions/mb/your_tool.rs`

### Step 2: Define Parameters

```rust
use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for your tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct YourToolParams {
    /// Description of parameter 1
    pub param1: String,

    /// Optional parameter with default
    #[serde(default)]
    pub param2: bool,

    /// Optional parameter (None if not provided)
    pub param3: Option<i32>,
}
```

**Key Points**:
- Must derive `Debug`, `Clone`, `Deserialize`, and `JsonSchema`
- Use `#[serde(default)]` for optional parameters with defaults
- Use `Option<T>` for truly optional parameters
- Add doc comments (they become part of the JSON schema)

### Step 3: Create Tool Struct

```rust
use rmcp::{
    ErrorData as McpError,
    handler::server::tool::{ToolRoute, ToolCallContext, cached_schema_for_type},
    model::{CallToolResult, Content, Tool},
};
use tracing::{info, warn, instrument};
use futures::FutureExt;

/// Your tool - brief description.
pub struct YourTool;

impl YourTool {
    /// Tool name as registered in MCP.
    pub const NAME: &'static str = "your_tool_name";

    /// Tool description shown to clients.
    pub const DESCRIPTION: &'static str =
        "Detailed description of what your tool does and how to use it.";
}
```

### Step 4: Implement execute() Method

```rust
impl YourTool {
    /// Execute the tool logic (for STDIO/TCP transport via rmcp).
    #[instrument(skip_all, fields(param1 = %params.param1))]
    pub fn execute(params: &YourToolParams) -> CallToolResult {
        info!("Your tool called with param1: {}", params.param1);

        // Validate inputs
        if params.param1.is_empty() {
            return CallToolResult::error(vec![
                Content::text("param1 cannot be empty")
            ]);
        }

        // Execute business logic
        let result = match your_business_logic(params) {
            Ok(data) => data,
            Err(e) => {
                warn!("Tool execution failed: {}", e);
                return CallToolResult::error(vec![
                    Content::text(format!("Error: {}", e))
                ]);
            }
        };

        // Return success
        CallToolResult::success(vec![Content::text(result)])
    }
}

fn your_business_logic(params: &YourToolParams) -> Result<String, String> {
    // Your implementation here
    Ok(format!("Processed: {}", params.param1))
}
```

**Key Points**:
- Use `#[instrument]` for tracing
- Validate inputs early
- Return `CallToolResult::error()` for errors
- Return `CallToolResult::success()` for success
- Use `Content::text()` for text responses
- **See [Tool Output Formats Guide](../reference/tool-output-formats.md)** for complete output format reference

### Step 5: Implement HTTP Handler (Optional)

```rust
impl YourTool {
    /// HTTP handler for this tool (for HTTP transport).
    #[cfg(feature = "http")]
    pub fn http_handler(arguments: serde_json::Value) -> Result<serde_json::Value, String> {
        // Parse required parameters
        let param1 = arguments
            .get("param1")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing or invalid 'param1' parameter".to_string())?
            .to_string();

        // Parse optional parameters
        let param2 = arguments
            .get("param2")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let param3 = arguments
            .get("param3")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        info!("Your tool (HTTP) called");

        // Build params struct
        let params = YourToolParams {
            param1,
            param2,
            param3,
        };

        // Execute core logic
        let result = Self::execute(&params);

        // Return JSON response
        Ok(serde_json::json!({
            "content": result.content,
            "isError": result.is_error.unwrap_or(false)
        }))
    }
}
```

**Key Points**:
- Only needed if HTTP transport support is required
- Gate with `#[cfg(feature = "http")]`
- Parse parameters manually from JSON
- Reuse `execute()` method for consistency
- Return structured JSON response

### Step 6: Implement Metadata Methods

```rust
impl YourTool {
    /// Create a Tool model for this tool (metadata).
    pub fn to_tool() -> Tool {
        Tool {
            name: Self::NAME.into(),
            description: Some(Self::DESCRIPTION.into()),
            input_schema: cached_schema_for_type::<YourToolParams>(),
            annotations: None,
            output_schema: None,
            icons: None,
            meta: None,
            title: None,
        }
    }

    /// Create a ToolRoute for STDIO/TCP transport.
    pub fn create_route<S>() -> ToolRoute<S>
    where
        S: Send + Sync + 'static,
    {
        ToolRoute::new_dyn(Self::to_tool(), |ctx: ToolCallContext<'_, S>| {
            let args = ctx.arguments.clone().unwrap_or_default();
            async move {
                let params: YourToolParams = serde_json::from_value(
                    serde_json::Value::Object(args)
                ).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                Ok(Self::execute(&params))
            }.boxed()
        })
    }
}
```

### Step 7: Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_success() {
        let params = YourToolParams {
            param1: "test".to_string(),
            param2: true,
            param3: Some(42),
        };

        let result = YourTool::execute(&params);
        assert!(result.is_error.is_none() || !result.is_error.unwrap());
    }

    #[test]
    fn test_execute_empty_param() {
        let params = YourToolParams {
            param1: "".to_string(),
            param2: false,
            param3: None,
        };

        let result = YourTool::execute(&params);
        assert!(result.is_error.unwrap_or(false));
    }

    #[cfg(feature = "http")]
    #[test]
    fn test_http_handler() {
        let args = serde_json::json!({
            "param1": "test",
            "param2": true
        });

        let result = YourTool::http_handler(args);
        assert!(result.is_ok());
    }
}
```

### Step 8: Register Tool

**8a. Add to mod.rs**

Add to `src/domains/tools/definitions/fs/mod.rs` (or appropriate module):

```rust
mod your_tool;
pub use your_tool::{YourTool, YourToolParams};
```

**8b. Register in registry.rs**

Add to `src/domains/tools/registry.rs`:

```rust
use super::definitions::fs::{YourTool};  // Add to imports

pub fn register_tools(registry: &mut ToolRegistry) {
    // ... existing registrations ...

    registry.register_tool(
        YourTool::NAME,
        YourTool::http_handler
    );
}
```

**8c. Add route in router.rs**

Add to `src/domains/tools/router.rs`:

```rust
use super::definitions::fs::YourTool;  // Add to imports

pub fn build_tool_router() -> ToolRouter<()> {
    ToolRouter::new()
        // ... existing routes ...
        .with_route(YourTool::create_route())
}
```

---

## Complete Example

Here's a minimal but complete tool:

```rust
//! Hello tool - returns a greeting message.

use rmcp::{
    ErrorData as McpError,
    handler::server::tool::{ToolRoute, ToolCallContext, cached_schema_for_type},
    model::{CallToolResult, Content, Tool},
};
use schemars::JsonSchema;
use serde::Deserialize;
use tracing::{info, instrument};
use futures::FutureExt;

// Parameters
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct HelloParams {
    /// Name to greet
    pub name: String,
}

// Tool
pub struct HelloTool;

impl HelloTool {
    pub const NAME: &'static str = "hello";
    pub const DESCRIPTION: &'static str = "Returns a friendly greeting message.";

    #[instrument(skip_all, fields(name = %params.name))]
    pub fn execute(params: &HelloParams) -> CallToolResult {
        info!("Hello tool called");
        let message = format!("Hello, {}! 👋", params.name);
        CallToolResult::success(vec![Content::text(message)])
    }

    #[cfg(feature = "http")]
    pub fn http_handler(arguments: serde_json::Value) -> Result<serde_json::Value, String> {
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'name' parameter".to_string())?
            .to_string();

        let params = HelloParams { name };
        let result = Self::execute(&params);

        Ok(serde_json::json!({
            "content": result.content,
            "isError": result.is_error.unwrap_or(false)
        }))
    }

    pub fn to_tool() -> Tool {
        Tool {
            name: Self::NAME.into(),
            description: Some(Self::DESCRIPTION.into()),
            input_schema: cached_schema_for_type::<HelloParams>(),
            annotations: None,
            output_schema: None,
            icons: None,
            meta: None,
            title: None,
        }
    }

    pub fn create_route<S>() -> ToolRoute<S>
    where
        S: Send + Sync + 'static,
    {
        ToolRoute::new_dyn(Self::to_tool(), |ctx: ToolCallContext<'_, S>| {
            let args = ctx.arguments.clone().unwrap_or_default();
            async move {
                let params: HelloParams = serde_json::from_value(
                    serde_json::Value::Object(args)
                ).map_err(|e| McpError::invalid_params(e.to_string(), None))?;
                Ok(Self::execute(&params))
            }.boxed()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        let params = HelloParams { name: "World".to_string() };
        let result = HelloTool::execute(&params);
        assert!(result.is_error.is_none());
    }
}
```

---

## Testing Your Tool

### Unit Tests

Run unit tests:

```bash
cargo test --features all
```

Test specific tool:

```bash
cargo test --features all your_tool
```

### Integration Testing

Use Python test scripts in `scripts/`:

```python
# scripts/test_your_tool.py
import json
import subprocess

def test_your_tool_stdio():
    """Test via STDIO transport"""
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "your_tool_name",
            "arguments": {
                "param1": "test value"
            }
        }
    }

    proc = subprocess.Popen(
        ["cargo", "run"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    stdout, stderr = proc.communicate(
        input=json.dumps(request).encode()
    )

    response = json.loads(stdout.decode())
    assert "result" in response
    print("✅ Tool works via STDIO")

if __name__ == "__main__":
    test_your_tool_stdio()
```

### HTTP Testing

```bash
# Start HTTP server
cargo run --features http

# In another terminal
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "your_tool_name",
      "arguments": {"param1": "test"}
    }
  }'
```

---

## Common Pitfalls

### 1. Forgetting to Register Tool

**Symptom**: Tool doesn't appear in `tools/list`

**Solution**: Add to both `registry.rs` AND `router.rs`

### 2. Missing Feature Gates

**Symptom**: Compilation errors when building without HTTP feature

**Solution**: Use `#[cfg(feature = "http")]` on HTTP-specific code

### 3. Incorrect Parameter Parsing in HTTP Handler

**Symptom**: Tool works in STDIO but fails in HTTP

**Solution**: Ensure HTTP handler parses all parameters correctly. Consider adding extensive tests.

### 4. Not Handling Errors Gracefully

**Symptom**: Tool panics or returns cryptic errors

**Solution**: Always use `Result` types and return `CallToolResult::error()` with clear messages

### 5. Blocking Operations in Async Context

**Symptom**: Server hangs or becomes unresponsive

**Solution**: For HTTP transport, use `std::thread::spawn`. For STDIO/TCP, blocking is handled by rmcp.

### 6. Missing JsonSchema Derive

**Symptom**: `cached_schema_for_type` fails to compile

**Solution**: Ensure parameter struct derives `JsonSchema`

### 7. Inconsistent Naming

**Symptom**: Tool works but naming feels off

**Solution**: Follow naming conventions:
- Constants: `SCREAMING_SNAKE_CASE`
- Structs: `PascalCase`
- Functions: `snake_case`
- Tool names (in MCP): `snake_case` with prefix (`fs_`, `mb_`, etc.)

---

## Next Steps

- Read [Tool System Architecture](../architecture/tool-system.md) for deeper understanding
- See [existing tools](../tools/) for more examples
- Check [Testing Guide](testing.md) for comprehensive testing strategies
- Refer to [Error Handling](../reference/error-handling.md) for error patterns

---

## Questions?

If you encounter issues:

1. Check existing tools for reference patterns
2. Review [CLAUDE.md](../../CLAUDE.md) for core principles
3. Run tests with `cargo test --features all -- --nocapture` for debug output
4. Check logs with `RUST_LOG=debug cargo run --features all`
