# MCP Tool Output Formats - Complete Guide

This guide provides comprehensive documentation on output formats for Model Context Protocol (MCP) tools. It covers all available content types, best practices, and common patterns for structuring tool responses.

## Table of Contents

1. [Overview](#overview)
2. [Quick Decision Tree](#quick-decision-tree)
3. [Content Types Reference](#content-types-reference)
4. [Output Patterns](#output-patterns)
5. [Error Handling](#error-handling)
6. [Output Schemas](#output-schemas)
7. [Annotations](#annotations)
8. [Best Practices](#best-practices)
9. [Common Mistakes](#common-mistakes)
10. [Migration Guide](#migration-guide)

---

## Overview

MCP tools can return multiple types of content in their responses. Choosing the right format ensures efficient processing by AI agents and maintains compatibility across different clients.

### Key Principles

1. **Use appropriate content types** for different data formats
2. **Always provide text fallback** for structured content
3. **Define output schemas** for complex, structured data
4. **Use annotations** to provide metadata
5. **Handle errors gracefully** with actionable messages

---

## Quick Decision Tree

```
What kind of data are you returning?

├─ Simple confirmation/message
│  └─ Use: Text Content
│
├─ Structured/JSON data (objects, arrays)
│  └─ Use: Structured Content + Text Summary
│
├─ File contents with metadata
│  └─ Use: Embedded Resource
│
├─ Reference to external file/URL
│  └─ Use: Resource Link
│
├─ Image or diagram
│  └─ Use: Image Content (base64)
│
├─ Audio data
│  └─ Use: Audio Content (base64)
│
└─ Complex result with multiple components
   └─ Use: Multiple Content Items
```

---

## Content Types Reference

### 1. Text Content

**Use for:** Simple text responses, confirmations, error messages

**Structure:**
```json
{
  "type": "text",
  "text": "Operation completed successfully"
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "File renamed from 'old.txt' to 'new.txt'"
      }
    ],
    "isError": false
  }
}
```

**When to use:**
- ✅ Simple confirmations
- ✅ Status messages
- ✅ Error messages
- ✅ Human-readable summaries

**When NOT to use:**
- ❌ Structured data (use `structuredContent` instead)
- ❌ Large datasets
- ❌ Data requiring parsing

---

### 2. Structured Content

**Use for:** Machine-readable JSON data that AI agents need to parse and process

**Structure:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Found 3 users matching criteria"
    }
  ],
  "structuredContent": {
    "users": [
      {"id": 1, "name": "Alice", "role": "admin"},
      {"id": 2, "name": "Bob", "role": "user"},
      {"id": 3, "name": "Charlie", "role": "user"}
    ],
    "total": 3,
    "query": "role:user OR role:admin"
  }
}
```

**⚠️ Critical: Always Include Text Content**

For backward compatibility, always provide a text summary alongside structured content:

```json
{
  "content": [
    {
      "type": "text",
      "text": "Weather data for New York:\n- Temperature: 22.5°C\n- Conditions: Partly cloudy\n- Humidity: 65%"
    }
  ],
  "structuredContent": {
    "location": "New York",
    "temperature": 22.5,
    "temperatureUnit": "celsius",
    "conditions": "Partly cloudy",
    "humidity": 65
  }
}
```

**❌ Common Mistake - JSON in Text Field:**

```json
// DON'T DO THIS!
{
  "content": [
    {
      "type": "text",
      "text": "{\"temperature\": 22.5, \"conditions\": \"Partly cloudy\"}"
    }
  ]
}
```

**Why this is wrong:**
- Requires double parsing (once as JSON-RPC, once for the stringified JSON)
- Poor readability for humans
- No schema validation
- Loses type information

**✅ Correct Approach:**

```json
{
  "content": [
    {
      "type": "text",
      "text": "Temperature: 22.5°C, Conditions: Partly cloudy"
    }
  ],
  "structuredContent": {
    "temperature": 22.5,
    "conditions": "Partly cloudy"
  }
}
```

**🎯 Optimization: Concise Summary for Large Data**

For large hierarchical data (file trees, nested objects, etc.), use a **concise text summary** instead of duplicating the entire structure:

**❌ DON'T - Duplicates full hierarchy:**
```rust
// Using CallToolResult::structured() automatically includes JSON in text
let result = CallToolResult::structured(serde_json::to_value(&large_data)?);
// Results in: text = "{...entire JSON...}" + structuredContent = {...entire JSON...}
// 🔴 Response size: ~500 KB (duplicated)
```

**✅ DO - Concise summary:**
```rust
// Create custom CallToolResult with summary
CallToolResult {
    content: vec![Content::text("Found 491 directories and 509 files")],
    structured_content: Some(serde_json::to_value(&large_data)?),
    is_error: Some(false),
    meta: None,
}
// 🟢 Response size: ~250 KB (optimized)
```

**Example - Directory Listing:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Found 16 directories and 0 files in '/music'"
    }
  ],
  "structuredContent": {
    "path": "/music",
    "entries": [
      {
        "name": "Techno",
        "type": "directory",
        "children": [
          {"name": "track1.mp3", "type": "file"},
          {"name": "track2.mp3", "type": "file"}
        ]
      }
    ],
    "dir_count": 16,
    "file_count": 1000
  }
}
```

**Benefits:**
- ✅ **50% smaller responses** (no duplication)
- ✅ **Human-readable summary** in text field
- ✅ **Full structure** in structuredContent for AI parsing
- ✅ **Faster transmission** over network

**When to use:**
- ✅ Complex data structures (objects, arrays)
- ✅ Data requiring type preservation
- ✅ Results that AI agents will process
- ✅ Hierarchical data

**When NOT to use:**
- ❌ Simple text messages
- ❌ Binary data (use image/audio instead)
- ❌ File contents (use embedded resource instead)

---

### 3. Embedded Resource

**Use for:** Complete file contents with metadata, rich context data

**Structure:**
```json
{
  "type": "resource",
  "resource": {
    "uri": "file:///project/config.json",
    "mimeType": "application/json",
    "text": "{\n  \"version\": \"1.0\",\n  \"name\": \"myapp\"\n}",
    "annotations": {
      "audience": ["user", "assistant"],
      "priority": 0.8,
      "lastModified": "2025-12-18T10:30:00Z"
    }
  }
}
```

**Example - Source Code File:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Retrieved main.rs (125 lines)"
    },
    {
      "type": "resource",
      "resource": {
        "uri": "file:///project/src/main.rs",
        "mimeType": "text/x-rust",
        "text": "fn main() {\n    println!(\"Hello, world!\");\n}"
      }
    }
  ]
}
```

**Example - API Response:**
```json
{
  "type": "resource",
  "resource": {
    "uri": "https://api.example.com/v1/data/12345",
    "mimeType": "application/json",
    "text": "{\"id\": 12345, \"status\": \"active\"}",
    "annotations": {
      "lastModified": "2025-12-18T14:23:00Z"
    }
  }
}
```

**When to use:**
- ✅ Complete file contents
- ✅ API responses with context
- ✅ Documents requiring URI reference
- ✅ Data with modification timestamps

**Benefits over text:**
- URI provides context and reference
- MIME type enables correct rendering
- Annotations provide metadata
- Clients can cache by URI

---

### 4. Resource Link

**Use for:** References to external resources without embedding full content

**Structure:**
```json
{
  "type": "resource_link",
  "uri": "file:///project/logs/app.log",
  "name": "Application Log",
  "description": "Main application log file",
  "mimeType": "text/plain"
}
```

**Example - Generated File Reference:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Generated report successfully"
    },
    {
      "type": "resource_link",
      "uri": "file:///reports/sales-2025-12.pdf",
      "name": "sales-2025-12.pdf",
      "description": "December 2025 sales report",
      "mimeType": "application/pdf"
    }
  ]
}
```

**Example - External Documentation:**
```json
{
  "type": "resource_link",
  "uri": "https://docs.example.com/api/v2/reference",
  "name": "API Reference",
  "description": "Complete API documentation",
  "mimeType": "text/html"
}
```

**When to use:**
- ✅ Large files (don't embed in response)
- ✅ References to external URLs
- ✅ Pointers to generated artifacts
- ✅ Log files or database entries

**Difference from Embedded Resource:**
- No content included (just reference)
- Clients may fetch content separately
- Lower response payload
- Not guaranteed to be in `resources/list`

---

### 5. Image Content

**Use for:** Charts, diagrams, screenshots, generated images

**Structure:**
```json
{
  "type": "image",
  "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
  "mimeType": "image/png",
  "annotations": {
    "audience": ["user"],
    "priority": 0.9
  }
}
```

**Example - Generated Chart:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Sales trend chart for Q4 2025"
    },
    {
      "type": "image",
      "data": "base64-encoded-png-data...",
      "mimeType": "image/png"
    }
  ]
}
```

**Supported MIME types:**
- `image/png`
- `image/jpeg`
- `image/gif`
- `image/svg+xml`
- `image/webp`

**When to use:**
- ✅ Generated visualizations
- ✅ Screenshots
- ✅ Diagrams
- ✅ Charts and graphs

**Important:**
- Always base64-encode binary data
- Specify correct MIME type
- Consider file size limits
- Provide text description for accessibility

---

### 6. Audio Content

**Use for:** Audio recordings, generated speech, sound data

**Structure:**
```json
{
  "type": "audio",
  "data": "base64-encoded-audio-data...",
  "mimeType": "audio/wav"
}
```

**Example:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Generated audio pronunciation for 'bonjour'"
    },
    {
      "type": "audio",
      "data": "UklGRiQAAABXQVZFZm10...",
      "mimeType": "audio/wav"
    }
  ]
}
```

**Supported MIME types:**
- `audio/wav`
- `audio/mpeg`
- `audio/ogg`
- `audio/mp4`

---

### 7. Multiple Content Items

**Use for:** Rich responses combining multiple types of information

**Example - Analysis with Summary, Data, and Chart:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Sales Analysis Summary:\n- Total revenue: $125,430\n- Growth: +15% vs last month\n- Top product: Widget Pro"
    },
    {
      "type": "image",
      "data": "base64-chart-data...",
      "mimeType": "image/png"
    },
    {
      "type": "resource_link",
      "uri": "file:///reports/detailed-analysis.xlsx",
      "name": "detailed-analysis.xlsx",
      "description": "Complete sales breakdown",
      "mimeType": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    }
  ],
  "structuredContent": {
    "totalRevenue": 125430,
    "growthPercent": 15,
    "topProduct": {
      "name": "Widget Pro",
      "revenue": 45200,
      "units": 120
    }
  }
}
```

**When to use:**
- ✅ Complex analyses
- ✅ Reports with multiple formats
- ✅ Results requiring both summary and details
- ✅ Responses with visualizations and data

---

## Output Patterns

### Pattern 1: Simple Confirmation

**Use case:** Basic operation confirmations, status messages

**Example:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Successfully created backup of database 'production' (2.3 GB)"
    }
  ]
}
```

**Characteristics:**
- Single text content item
- Human-readable message
- No structured data needed

---

### Pattern 2: Structured Data Response

**Use case:** Search results, database queries, API data

**Example - User Search:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Found 5 active users in the system"
    }
  ],
  "structuredContent": {
    "users": [
      {"id": 1, "username": "alice", "role": "admin", "lastSeen": "2025-12-18T10:00:00Z"},
      {"id": 2, "username": "bob", "role": "user", "lastSeen": "2025-12-18T09:30:00Z"},
      {"id": 3, "username": "charlie", "role": "user", "lastSeen": "2025-12-17T15:20:00Z"},
      {"id": 4, "username": "diana", "role": "moderator", "lastSeen": "2025-12-18T11:45:00Z"},
      {"id": 5, "username": "eve", "role": "user", "lastSeen": "2025-12-18T08:15:00Z"}
    ],
    "totalCount": 5,
    "filters": {
      "status": "active"
    }
  }
}
```

**Characteristics:**
- Text summary for humans
- Structured data for AI processing
- Requires output schema (see below)

---

### Pattern 3: Hierarchical Data (Nested Structure)

**Use case:** Directory trees, organizational charts, nested categories

**Example - Directory Listing:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Directory structure for /project (15 files, 5 directories)"
    }
  ],
  "structuredContent": {
    "path": "/project",
    "entries": [
      {
        "name": "src",
        "type": "directory",
        "children": [
          {
            "name": "main.rs",
            "type": "file",
            "size": 1024
          },
          {
            "name": "lib.rs",
            "type": "file",
            "size": 2048
          }
        ]
      },
      {
        "name": "tests",
        "type": "directory",
        "children": [
          {
            "name": "integration_test.rs",
            "type": "file",
            "size": 512
          }
        ]
      },
      {
        "name": "Cargo.toml",
        "type": "file",
        "size": 256
      }
    ]
  }
}
```

**Characteristics:**
- Natural tree structure
- Easy navigation for AI
- No path redundancy
- Clear parent-child relationships

---

### Pattern 4: File Content with Context

**Use case:** Reading file contents, showing code, displaying configurations

**Example - Configuration File:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Configuration file (last modified: 2025-12-18)"
    },
    {
      "type": "resource",
      "resource": {
        "uri": "file:///app/config/settings.json",
        "mimeType": "application/json",
        "text": "{\n  \"port\": 8080,\n  \"host\": \"localhost\",\n  \"debug\": true\n}",
        "annotations": {
          "lastModified": "2025-12-18T14:30:00Z",
          "audience": ["user", "assistant"]
        }
      }
    }
  ]
}
```

**Characteristics:**
- URI provides context
- MIME type for proper rendering
- Metadata via annotations
- Full content embedded

---

### Pattern 5: Analysis with Visualization

**Use case:** Data analysis, reports, metrics with charts

**Example - Performance Analysis:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "API Performance Analysis:\n- Average response time: 145ms\n- 99th percentile: 320ms\n- Error rate: 0.02%\n- Throughput: 1,250 req/s"
    },
    {
      "type": "image",
      "data": "base64-encoded-chart...",
      "mimeType": "image/png",
      "annotations": {
        "audience": ["user"],
        "priority": 0.9
      }
    }
  ],
  "structuredContent": {
    "metrics": {
      "averageResponseMs": 145,
      "p99ResponseMs": 320,
      "errorRatePercent": 0.02,
      "throughputReqPerSec": 1250
    },
    "timeRange": {
      "start": "2025-12-18T00:00:00Z",
      "end": "2025-12-18T23:59:59Z"
    }
  }
}
```

**Characteristics:**
- Text summary
- Visual representation
- Structured metrics for processing
- Multiple content types

---

## Error Handling

### Actionable Errors

**Use for:** Errors that AI can understand and potentially fix

**Example - Validation Error:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Invalid date format: '2025/12/18'. Expected format: YYYY-MM-DD (e.g., 2025-12-18)"
    }
  ],
  "isError": true
}
```

**Example - Resource Not Found:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "File not found: '/data/config.json'. Available files in /data: database.db, settings.yaml"
    }
  ],
  "isError": true
}
```

**Characteristics:**
- Clear error description
- Actionable guidance
- Suggests corrections
- Returns in `result` with `isError: true`

---

### Protocol Errors

**Use for:** Structural problems, unknown tools, server failures

**Example - Unknown Tool:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32602,
    "message": "Unknown tool: 'nonexistent_tool'. Available tools: search, analyze, create"
  }
}
```

**Example - Invalid Parameters:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": -32602,
    "message": "Invalid params: missing required field 'query'"
  }
}
```

**Characteristics:**
- JSON-RPC error format
- Standard error codes
- Returns in `error` field
- Less likely to be self-correctable

---

### Error Best Practices

1. **Be Specific:**
   ```json
   // ❌ Bad
   {"text": "Error occurred"}

   // ✅ Good
   {"text": "Connection timeout after 30s connecting to database 'production' at host db.example.com:5432"}
   ```

2. **Provide Context:**
   ```json
   // ✅ Good
   {"text": "Permission denied writing to '/etc/config.json'. Current user: 'appuser'. Required permission: write. File owner: root"}
   ```

3. **Suggest Solutions:**
   ```json
   // ✅ Good
   {"text": "API rate limit exceeded (100 requests/hour). Limit resets at 2025-12-18T15:00:00Z. Consider reducing request frequency or upgrading plan."}
   ```

---

## Output Schemas

### Defining Output Schemas

Output schemas validate structured content and provide type information to clients.

**Example Tool Definition with Schema:**
```json
{
  "name": "search_users",
  "description": "Search for users by criteria",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": {"type": "string"}
    },
    "required": ["query"]
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "users": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "id": {"type": "integer"},
            "username": {"type": "string"},
            "email": {"type": "string", "format": "email"},
            "role": {"type": "string", "enum": ["admin", "user", "moderator"]},
            "createdAt": {"type": "string", "format": "date-time"}
          },
          "required": ["id", "username", "role"]
        }
      },
      "totalCount": {"type": "integer"},
      "hasMore": {"type": "boolean"}
    },
    "required": ["users", "totalCount"]
  }
}
```

### Benefits of Output Schemas

1. **Validation:** Ensures responses conform to expected structure
2. **Type Safety:** Provides type information for client-side processing
3. **Documentation:** Self-documenting tool outputs
4. **IDE Support:** Better autocomplete and type checking
5. **Error Prevention:** Catches schema violations early

### Schema Best Practices

1. **Mark Required Fields:**
   ```json
   {
     "required": ["id", "name", "status"]
   }
   ```

2. **Use Descriptive Types:**
   ```json
   {
     "timestamp": {
       "type": "string",
       "format": "date-time",
       "description": "ISO 8601 timestamp"
     }
   }
   ```

3. **Constrain Values:**
   ```json
   {
     "status": {
       "type": "string",
       "enum": ["pending", "active", "completed", "failed"]
     }
   }
   ```

4. **Document Fields:**
   ```json
   {
     "confidence": {
       "type": "number",
       "minimum": 0,
       "maximum": 1,
       "description": "Confidence score between 0 and 1"
     }
   }
   ```

---

## Annotations

All content types support optional annotations for metadata.

### Annotation Fields

```json
{
  "annotations": {
    "audience": ["user", "assistant"],
    "priority": 0.8,
    "lastModified": "2025-12-18T14:30:00Z",
    "customField": "customValue"
  }
}
```

### Audience Targeting

Specify who should see the content:

```json
// For user's eyes only
{"audience": ["user"]}

// For AI assistant only
{"audience": ["assistant"]}

// For both (default)
{"audience": ["user", "assistant"]}
```

**Example:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Database query completed in 145ms",
      "annotations": {
        "audience": ["user"]
      }
    },
    {
      "type": "resource",
      "resource": {
        "uri": "internal://debug/query-plan",
        "text": "SELECT ... EXPLAIN ANALYZE ...",
        "annotations": {
          "audience": ["assistant"]
        }
      }
    }
  ]
}
```

### Priority Levels

Indicate importance (0.0 to 1.0):

```json
// High priority
{"priority": 0.9}

// Normal priority
{"priority": 0.5}

// Low priority
{"priority": 0.2}
```

**Use cases:**
- Highlight critical warnings
- Deprioritize verbose debug info
- Focus AI attention on key results

### Timestamps

Track modification times:

```json
{
  "annotations": {
    "lastModified": "2025-12-18T14:30:00Z",
    "createdAt": "2025-12-15T10:00:00Z"
  }
}
```

**Benefits:**
- Cache invalidation
- Freshness tracking
- Audit trails

---

## Best Practices

### Decision Matrix

| Data Type | Recommended Format | Why |
|-----------|-------------------|-----|
| Simple message | Text | Human-readable, no parsing needed |
| JSON object/array | Structured Content + Text | Type-safe, validated, parseable |
| File contents | Embedded Resource | Context (URI), metadata, MIME type |
| External reference | Resource Link | Avoids large payloads |
| Image/chart | Image Content | Visual data requires special handling |
| Audio | Audio Content | Binary format with MIME type |
| Complex report | Multiple Content | Combine summary + data + visuals |

### Performance Considerations

1. **Payload Size:**
   - Text: ~1 KB per message
   - Structured: ~5-50 KB typical
   - Images: 50 KB - 5 MB (use compression)
   - Large files: Use Resource Link, not embedded

2. **Parsing Efficiency:**
   - Structured content: Single parse
   - JSON in text: Double parse (avoid!)

3. **Caching:**
   - Use Resource URIs for cacheable content
   - Include `lastModified` annotations

4. **Avoid Duplication (Critical for Large Data):**
   - ❌ **Don't use `CallToolResult::structured()`** for large hierarchical data
   - ✅ **Do create custom `CallToolResult`** with concise text summary
   - **Example:** Directory listing with 1000 files
     - With duplication: ~500 KB response
     - With summary: ~250 KB response (50% reduction)

**Implementation Example (Rust/rmcp):**

```rust
// ❌ BAD - Duplicates entire hierarchy in text field
pub fn execute(params: &Params) -> CallToolResult {
    let large_data = build_hierarchy(); // 1000 entries
    CallToolResult::structured(serde_json::to_value(&large_data).unwrap())
    // Result: text = "{\"entries\":[...]}" (full JSON)
    //         + structuredContent = {...} (same JSON)
}

// ✅ GOOD - Concise summary + full structure
pub fn execute(params: &Params) -> CallToolResult {
    let large_data = build_hierarchy();

    // Create human-readable summary
    let summary = format!(
        "Found {} directories and {} files in '{}'",
        large_data.dir_count,
        large_data.file_count,
        params.path
    );

    // Return both summary + structured data
    CallToolResult {
        content: vec![Content::text(summary)],
        structured_content: Some(serde_json::to_value(&large_data).unwrap()),
        is_error: Some(false),
        meta: None,
    }
}
```

**When to optimize:**
- ✅ File/directory trees (>100 entries)
- ✅ Large nested objects (>10 KB)
- ✅ Database query results (>50 rows)
- ✅ API responses with arrays

**When duplication is OK:**
- ✅ Small objects (<5 KB)
- ✅ Simple data (2-3 fields)
- ✅ Non-hierarchical data

### Compatibility Guidelines

1. **Always Provide Text:**
   ```json
   // ✅ Good - works everywhere
   {
     "content": [{"type": "text", "text": "..."}],
     "structuredContent": {...}
   }
   ```

2. **Graceful Degradation:**
   - Clients without structured content support fall back to text
   - Older clients ignore unknown fields

3. **Schema Evolution:**
   - Add optional fields (don't remove required)
   - Version schemas if breaking changes needed

### Security Considerations

1. **Sanitize URIs:**
   ```json
   // Validate URIs before returning
   "uri": "file:///safe/path/file.txt"
   ```

2. **Limit Payload Size:**
   - Prevent DoS via huge responses
   - Set max limits (e.g., 10 MB)

3. **Sensitive Data:**
   ```json
   // Use audience to restrict
   {
     "annotations": {
       "audience": ["assistant"],  // Hide from user
       "sensitivity": "high"
     }
   }
   ```

---

## Common Mistakes

### ❌ Mistake 1: JSON in Text Field

```json
// DON'T DO THIS
{
  "content": [
    {
      "type": "text",
      "text": "{\"users\": [{\"id\": 1, \"name\": \"Alice\"}]}"
    }
  ]
}
```

**Why it's wrong:**
- Requires double parsing
- No type safety
- No schema validation
- Poor readability

**✅ Correct:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Found 1 user: Alice (ID: 1)"
    }
  ],
  "structuredContent": {
    "users": [{"id": 1, "name": "Alice"}]
  }
}
```

---

### ❌ Mistake 2: Missing Text Fallback

```json
// DON'T DO THIS
{
  "structuredContent": {
    "temperature": 22.5
  }
  // No text content!
}
```

**Why it's wrong:**
- Breaks backward compatibility
- Non-supporting clients see nothing
- Poor user experience

**✅ Correct:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Temperature: 22.5°C"
    }
  ],
  "structuredContent": {
    "temperature": 22.5,
    "unit": "celsius"
  }
}
```

---

### ❌ Mistake 3: Wrong Content Type

```json
// DON'T DO THIS - using text for binary data
{
  "content": [
    {
      "type": "text",
      "text": "iVBORw0KGgoAAAANSUhEUg..."  // Image as text
    }
  ]
}
```

**✅ Correct:**
```json
{
  "content": [
    {
      "type": "image",
      "data": "iVBORw0KGgoAAAANSUhEUg...",
      "mimeType": "image/png"
    }
  ]
}
```

---

### ❌ Mistake 4: Flat List Instead of Hierarchy

For hierarchical data like directories:

```json
// AVOID - loses structure
{
  "structuredContent": {
    "entries": [
      {"path": "src", "type": "directory"},
      {"path": "src/main.rs", "type": "file"},
      {"path": "src/lib.rs", "type": "file"},
      {"path": "tests", "type": "directory"},
      {"path": "tests/test.rs", "type": "file"}
    ]
  }
}
```

**✅ Better - preserves hierarchy:**
```json
{
  "structuredContent": {
    "entries": [
      {
        "name": "src",
        "type": "directory",
        "children": [
          {"name": "main.rs", "type": "file"},
          {"name": "lib.rs", "type": "file"}
        ]
      },
      {
        "name": "tests",
        "type": "directory",
        "children": [
          {"name": "test.rs", "type": "file"}
        ]
      }
    ]
  }
}
```

**When to use flat vs nested:**
- Flat: 1000+ items, prevents deep nesting
- Nested: < 1000 items, natural structure

---

### ❌ Mistake 5: Missing Schema Definition

```json
// Tool definition without output schema
{
  "name": "get_weather",
  "inputSchema": {...}
  // No outputSchema!
}
```

**Why it's wrong:**
- No validation
- Unclear structure
- Type ambiguity

**✅ Correct:**
```json
{
  "name": "get_weather",
  "inputSchema": {...},
  "outputSchema": {
    "type": "object",
    "properties": {
      "temperature": {"type": "number"},
      "conditions": {"type": "string"}
    },
    "required": ["temperature", "conditions"]
  }
}
```

---

### ❌ Mistake 6: Vague Error Messages

```json
// DON'T DO THIS
{
  "content": [
    {
      "type": "text",
      "text": "Error"
    }
  ],
  "isError": true
}
```

**✅ Correct:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Database connection failed: timeout after 30s connecting to postgres://db.example.com:5432/production. Check network connectivity and database status."
    }
  ],
  "isError": true
}
```

---

## Migration Guide

### From Text-Only to Structured Content

**Before (text-only):**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Users: Alice (admin), Bob (user), Charlie (user)"
    }
  ]
}
```

**After (structured + text):**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Found 3 users: 1 admin, 2 regular users"
    }
  ],
  "structuredContent": {
    "users": [
      {"name": "Alice", "role": "admin"},
      {"name": "Bob", "role": "user"},
      {"name": "Charlie", "role": "user"}
    ],
    "summary": {
      "total": 3,
      "byRole": {
        "admin": 1,
        "user": 2
      }
    }
  }
}
```

**Migration steps:**
1. Add `outputSchema` to tool definition
2. Keep existing text format for compatibility
3. Add `structuredContent` field with parsed data
4. Test with both old and new clients
5. Update documentation

### Backward Compatibility Strategy

```rust
// Example: Support both formats during transition
fn create_response(data: &Data) -> ToolResult {
    ToolResult {
        content: vec![
            // Legacy text format (always include)
            Content::text(format_as_text(data))
        ],
        // New structured format (opt-in)
        structured_content: Some(data.to_json()),
        ..Default::default()
    }
}
```

---

## References

### Official Specification
- [MCP Tools Specification](https://modelcontextprotocol.io/specification/2025-06-18/server/tools)
- [MCP Complete Guide 2025](https://www.keywordsai.co/blog/introduction-to-mcp)

### Example Implementations
- [MCP Reference Servers](https://github.com/modelcontextprotocol/servers)
- [GitHub MCP Server](https://github.com/github/github-mcp-server)
- [Building MCP Servers Guide](https://github.blog/ai-and-ml/github-copilot/building-your-first-mcp-server-how-to-extend-ai-tools-with-custom-capabilities/)

### Related Documentation
- [Adding New Tools](../guides/adding-tools.md)
- [Error Handling](error-handling.md)
- [Testing Strategy](../guides/testing.md)

---

## Summary Checklist

When implementing a tool output:

- [ ] Choose appropriate content type(s)
- [ ] Always include text content for structured data
- [ ] Define output schema for complex data
- [ ] Use annotations for metadata
- [ ] Handle errors with actionable messages
- [ ] Validate against schema before returning
- [ ] Consider backward compatibility
- [ ] Document expected output format
- [ ] Test with multiple clients
- [ ] Review for common mistakes

---

**Last Updated:** 2025-12-18

**Version:** 1.0.0
