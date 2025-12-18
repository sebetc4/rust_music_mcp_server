# Tool System Architecture

This document explains how the tool system works in the Music MCP Server, with special focus on the **dual handler pattern** that enables support for multiple transport layers.

---

## Table of Contents

1. [Overview](#overview)
2. [Tool Lifecycle](#tool-lifecycle)
3. [Dual Handler Pattern](#dual-handler-pattern)
4. [Registration Flow](#registration-flow)
5. [Parameter Validation](#parameter-validation)
6. [Error Handling](#error-handling)

---

## Overview

The tool system is the core of the MCP server's functionality. It provides a **unified interface** for tools while supporting **three different transport layers**: STDIO, TCP, and HTTP.

### Key Design Goals

- **Consistency**: Same tool logic works across all transports
- **Flexibility**: Easy to add new tools without modifying infrastructure
- **Type Safety**: Strong typing with compile-time validation
- **Testability**: Tools can be tested independently of transport

### Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                     Tool Interface                       │
│  (CallToolResult, Content, Tool metadata)                │
└─────────────────────────────────────────────────────────┘
                           │
        ┌──────────────────┴──────────────────┐
        │                                      │
┌───────▼───────┐                    ┌────────▼────────┐
│  HTTP Handler  │                    │  STDIO/TCP      │
│  (registry.rs) │                    │  (router.rs)    │
└───────┬────────┘                    └────────┬────────┘
        │                                      │
        │         ┌────────────────────┐       │
        └────────►│  execute() method  │◄──────┘
                  │  (core logic)      │
                  └────────────────────┘
                           │
                  ┌────────▼────────┐
                  │  Business Logic  │
                  │  External APIs   │
                  │  File Operations │
                  └──────────────────┘
```

---

## Tool Lifecycle

### 1. Tool Definition Phase (Compile Time)

```rust
// Define parameters
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FsListDirParams {
    pub path: String,
    #[serde(default)]
    pub detailed: bool,
}

// Define tool
pub struct FsListDirTool;

impl FsListDirTool {
    pub const NAME: &'static str = "fs_list_dir";
    pub const DESCRIPTION: &'static str = "List directory contents";
}
```

### 2. Registration Phase (Startup)

```
Server Startup
     │
     ├─► HTTP Transport?
     │   └─► Register in ToolRegistry (registry.rs)
     │       └─► Map "fs_list_dir" → http_handler()
     │
     └─► STDIO/TCP Transport?
         └─► Build ToolRouter (router.rs)
             └─► Add ToolRoute for "fs_list_dir"
```

### 3. Execution Phase (Runtime)

#### STDIO/TCP Path

```
Client Request (JSON-RPC)
     │
     ▼
ToolRouter receives call
     │
     ▼
Lookup tool by name
     │
     ▼
Deserialize arguments to Params struct
     │
     ▼
Call execute(params)
     │
     ▼
Return CallToolResult
     │
     ▼
Serialize to JSON-RPC response
```

#### HTTP Path

```
HTTP POST /mcp
     │
     ▼
Axum routes to tools/call handler
     │
     ▼
ToolRegistry lookup by name
     │
     ▼
Call http_handler(arguments)
     │
     ├─► Parse JSON arguments manually
     │
     ├─► Build Params struct
     │
     ├─► Call execute(params)
     │
     └─► Convert CallToolResult to JSON
     │
     ▼
Return HTTP response
```

---

## Dual Handler Pattern

### Why Two Handlers?

The server supports fundamentally different execution models:

| Aspect | STDIO/TCP | HTTP |
|--------|-----------|------|
| **Runtime** | Tokio async | Blocking threads |
| **Blocking calls** | `tokio::spawn_blocking` | `std::thread::spawn` |
| **Parameter parsing** | rmcp handles it | Manual parsing |
| **Response format** | `CallToolResult` | JSON Value |

**Problem**: A single handler can't efficiently support both models.

**Solution**: Dual handler pattern - two handlers, shared core logic.

### Pattern Structure

```rust
pub struct MyTool;

impl MyTool {
    /// Core business logic (shared)
    pub fn execute(params: &MyToolParams) -> CallToolResult {
        // 1. Validate inputs
        // 2. Execute logic
        // 3. Return result
        CallToolResult::success(vec![Content::text(result)])
    }

    /// HTTP-specific handler (HTTP only)
    #[cfg(feature = "http")]
    pub fn http_handler(arguments: serde_json::Value)
        -> Result<serde_json::Value, String>
    {
        // 1. Parse JSON arguments manually
        let param1 = arguments.get("param1")...;

        // 2. Build params struct
        let params = MyToolParams { param1, ... };

        // 3. Call shared execute()
        let result = Self::execute(&params);

        // 4. Convert to JSON
        Ok(serde_json::json!({
            "content": result.content,
            "isError": result.is_error.unwrap_or(false)
        }))
    }

    /// STDIO/TCP route creator
    pub fn create_route<S>() -> ToolRoute<S> {
        ToolRoute::new_dyn(Self::to_tool(), |ctx| {
            let args = ctx.arguments.clone().unwrap_or_default();
            async move {
                // 1. Deserialize params (automatic)
                let params = serde_json::from_value(
                    serde_json::Value::Object(args)
                )?;

                // 2. Call shared execute()
                Ok(Self::execute(&params))
            }.boxed()
        })
    }
}
```

### Code Reuse

```
                  execute()
                     │
                     │ (shared core logic)
                     │
        ┌────────────┴────────────┐
        │                         │
   http_handler()          create_route()
        │                         │
        │                         │
  HTTP Transport            STDIO/TCP Transport
```

**Key Insight**: `execute()` contains **all business logic**. Handlers are thin wrappers for transport-specific concerns.

### Threading Model

#### HTTP Transport

```rust
#[cfg(feature = "http")]
pub fn http_handler(arguments: serde_json::Value)
    -> Result<serde_json::Value, String>
{
    // This runs in a dedicated std::thread
    // spawned by Axum for each request

    let result = blocking_external_api_call(); // OK!

    Ok(result)
}
```

- Runs in `std::thread` (spawned per request)
- Blocking calls are **safe and expected**
- No async context to worry about

#### STDIO/TCP Transport

```rust
pub fn create_route<S>() -> ToolRoute<S> {
    ToolRoute::new_dyn(Self::to_tool(), |ctx| {
        async move {
            let params = parse(ctx.arguments)?;

            // execute() is sync but called from async context
            // rmcp handles this internally
            Ok(Self::execute(&params))
        }.boxed()
    })
}
```

- Runs in Tokio async runtime
- rmcp SDK handles blocking operations internally
- No need for manual `spawn_blocking` in tool code

### Real-World Example

From `mb/identify_record.rs`:

```rust
impl MbIdentifyRecordTool {
    // Core logic (600+ lines)
    pub fn execute(params: &MbIdentifyRecordParams) -> CallToolResult {
        // 1. Generate fingerprint (blocking: fpcalc)
        let fingerprint = generate_fingerprint(&params.file_path)?;

        // 2. Query AcoustID API (blocking: HTTP request)
        let matches = query_acoustid(&fingerprint, &params.metadata_level)?;

        // 3. Enrich with MusicBrainz (blocking: multiple HTTP requests)
        let enriched = enrich_matches(matches, &params.metadata_level)?;

        // 4. Format and return
        CallToolResult::success(vec![Content::text(format_results(enriched))])
    }

    // HTTP handler (50 lines)
    #[cfg(feature = "http")]
    pub fn http_handler(args: serde_json::Value)
        -> Result<serde_json::Value, String>
    {
        // Parse JSON parameters
        let file_path = args.get("file_path")...;
        let metadata_level = parse_metadata_level(args)?;

        // Build params
        let params = MbIdentifyRecordParams { file_path, metadata_level };

        // Reuse core logic
        let result = Self::execute(&params);

        // Return JSON
        Ok(json!({"content": result.content, ...}))
    }

    // STDIO/TCP route (30 lines)
    pub fn create_route<S>() -> ToolRoute<S> {
        ToolRoute::new_dyn(Self::to_tool(), |ctx| {
            async move {
                let params: MbIdentifyRecordParams =
                    serde_json::from_value(...)?;
                Ok(Self::execute(&params))
            }.boxed()
        })
    }
}
```

**Result**: 600 lines of logic + 80 lines of transport glue = 680 total lines, **not** 1200+ lines (if logic was duplicated).

---

## Registration Flow

### HTTP Registration (registry.rs)

```rust
// src/domains/tools/registry.rs

use std::collections::HashMap;
use serde_json::Value;

pub struct ToolRegistry {
    tools: HashMap<
        String,
        Box<dyn Fn(Value) -> Result<Value, String> + Send + Sync>
    >,
}

pub fn register_tools(registry: &mut ToolRegistry) {
    // Register each tool's HTTP handler
    registry.register_tool(
        FsListDirTool::NAME,
        FsListDirTool::http_handler
    );

    registry.register_tool(
        ReadMetadataTool::NAME,
        ReadMetadataTool::http_handler
    );

    // ... etc
}
```

**Key Points**:
- HashMap lookup by tool name (String)
- Function pointers to HTTP handlers
- Used by HTTP transport only

### STDIO/TCP Registration (router.rs)

```rust
// src/domains/tools/router.rs

use rmcp::handler::server::tool::ToolRouter;

pub fn build_tool_router() -> ToolRouter<()> {
    ToolRouter::new()
        .with_route(FsListDirTool::create_route())
        .with_route(FsRenameTool::create_route())
        .with_route(ReadMetadataTool::create_route())
        .with_route(WriteMetadataTool::create_route())
        .with_route(MbArtistSearchTool::create_route())
        .with_route(MbReleaseSearchTool::create_route())
        .with_route(MbRecordingSearchTool::create_route())
        .with_route(MbAdvancedSearchTool::create_route())
        .with_route(MbIdentifyRecordTool::create_route())
}
```

**Key Points**:
- Builder pattern for composing routes
- Each route created by tool's `create_route()` method
- Used by STDIO/TCP transports only

### Consistency Check Test

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_registry_router_consistency() {
        let registry_tools = get_registry_tool_names();
        let router_tools = get_router_tool_names();

        assert_eq!(
            registry_tools, router_tools,
            "Registry and Router must have the same tools"
        );
    }
}
```

This test ensures tools aren't accidentally registered in only one place.

---

## Parameter Validation

### Type-Level Validation

```rust
#[derive(Deserialize, JsonSchema)]
pub struct MbArtistSearchParams {
    /// Artist name to search for (required)
    pub artist: String,

    /// Maximum number of results (1-100)
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 { 25 }
```

**What's validated automatically**:
- Field presence (missing required fields → error)
- Type correctness (string where number expected → error)
- JSON structure (extra fields ignored, nested structures parsed)

### Runtime Validation

```rust
pub fn execute(params: &MbArtistSearchParams) -> CallToolResult {
    // Validate business rules
    if params.artist.trim().is_empty() {
        return CallToolResult::error(vec![
            Content::text("Artist name cannot be empty")
        ]);
    }

    // Clamp limit to valid range
    let limit = params.limit.clamp(1, 100);

    // Proceed with validated data
    search_artist(&params.artist, limit)
}
```

**What needs manual validation**:
- Business logic constraints (non-empty strings, value ranges)
- Cross-field dependencies (if A is set, B must be set)
- External resource validation (file exists, URL reachable)

### Common Validation Pattern

```rust
// src/domains/tools/definitions/mb/common.rs

/// Validate that a string is a valid MusicBrainz ID (UUID format)
pub fn is_mbid(s: &str) -> bool {
    s.len() == 36
        && s.chars().filter(|c| *c == '-').count() == 4
        && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Validate and clamp limit parameter
pub fn validate_limit(limit: u32) -> u32 {
    limit.clamp(1, 100)
}
```

Shared validation utilities in `common.rs` promote consistency.

---

## Error Handling

### Error Types

```rust
// Success
CallToolResult::success(vec![Content::text("Result data")])

// Error
CallToolResult::error(vec![Content::text("Error message")])

// With structured error
CallToolResult {
    content: vec![Content::text("Detailed error info")],
    is_error: Some(true),
}
```

### Error Patterns

#### Pattern 1: Early Return

```rust
pub fn execute(params: &Params) -> CallToolResult {
    if !validate(params) {
        return CallToolResult::error(vec![
            Content::text("Validation failed")
        ]);
    }

    let data = match fetch_data(params) {
        Ok(d) => d,
        Err(e) => return CallToolResult::error(vec![
            Content::text(format!("Fetch failed: {}", e))
        ]),
    };

    CallToolResult::success(vec![Content::text(data)])
}
```

#### Pattern 2: Result Chaining

```rust
pub fn execute(params: &Params) -> CallToolResult {
    let result = validate(params)
        .and_then(|_| fetch_data(params))
        .and_then(|data| process(data))
        .map(|processed| format_output(processed));

    match result {
        Ok(output) => CallToolResult::success(vec![Content::text(output)]),
        Err(e) => CallToolResult::error(vec![Content::text(e.to_string())]),
    }
}
```

#### Pattern 3: Detailed Error Context

```rust
pub fn execute(params: &Params) -> CallToolResult {
    match dangerous_operation(params) {
        Ok(result) => CallToolResult::success(vec![
            Content::text(result)
        ]),
        Err(e) => {
            warn!("Operation failed: {:?}", e);
            CallToolResult::error(vec![Content::text(format!(
                "Operation failed: {}\n\nTroubleshooting:\n\
                 - Check that the file exists\n\
                 - Ensure you have read permissions\n\
                 - Verify the file format is supported",
                e
            ))])
        }
    }
}
```

---

## Summary

### Key Takeaways

1. **Dual Handler Pattern** enables support for multiple transports without code duplication
2. **Core logic in `execute()`** is transport-agnostic and reusable
3. **HTTP handler** does manual parameter parsing and runs in threads
4. **STDIO/TCP routes** use rmcp's automatic deserialization and async handling
5. **Registration happens twice** - once in registry.rs (HTTP), once in router.rs (STDIO/TCP)
6. **Type-level and runtime validation** work together for robust input handling
7. **CallToolResult** provides a unified error/success interface

### Adding a New Tool Checklist

- [ ] Implement parameter struct with `Deserialize` + `JsonSchema`
- [ ] Implement `execute()` method with core logic
- [ ] Implement `http_handler()` (if HTTP support needed)
- [ ] Implement `to_tool()` for metadata
- [ ] Implement `create_route()` for STDIO/TCP
- [ ] Add to `mod.rs` exports
- [ ] Register in `registry.rs` (HTTP)
- [ ] Register in `router.rs` (STDIO/TCP)
- [ ] Write unit tests
- [ ] Test with all transports

---

## Related Documentation

- [Adding New Tools](../guides/adding-tools.md) - Step-by-step tutorial
- [Transport Layer](transport-layer.md) - How STDIO/TCP/HTTP transports work
- [Error Handling](../reference/error-handling.md) - Deep dive into error patterns
- [Testing Guide](../guides/testing.md) - How to test tools effectively
