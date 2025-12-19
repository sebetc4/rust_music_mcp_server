# MCP Tool Output - Quick Reference Card

> **Full Guide:** See [tool-output-formats.md](tool-output-formats.md) for complete documentation

## Quick Decision Tree

```
Return simple message? → Text Content
Return JSON data? → Structured Content + Text Summary
Return file content? → Embedded Resource
Reference external file? → Resource Link
Return image/chart? → Image Content
Return audio? → Audio Content
Complex response? → Multiple Content Items
```

## Common Patterns

### ✅ Simple Text Response
```json
{
  "content": [{"type": "text", "text": "Operation completed"}]
}
```

### ✅ Structured Data (RECOMMENDED for JSON)
```json
{
  "content": [{"type": "text", "text": "Found 3 users"}],
  "structuredContent": {
    "users": [{"id": 1, "name": "Alice"}],
    "total": 3
  }
}
```

### ✅ Large Hierarchical Data (OPTIMIZED)
```json
{
  "content": [{"type": "text", "text": "Found 16 dirs and 1000 files"}],
  "structuredContent": {
    "entries": [
      {"name": "dir1", "type": "directory", "children": [...]}
    ],
    "dir_count": 16,
    "file_count": 1000
  }
}
```
**Benefits:** 50% smaller (no duplication), human-readable summary

### ❌ WRONG: JSON in Text
```json
// DON'T DO THIS!
{
  "content": [{"type": "text", "text": "{\"users\": [...]}"}]
}
```

### ❌ WRONG: Duplicating Large Data
```json
// DON'T DO THIS for large data!
{
  "content": [{"type": "text", "text": "{\"entries\":[...1000 items...]}"}],
  "structuredContent": {"entries": [...1000 items...]}
}
```
**Problem:** Response 2× larger, wastes bandwidth

## Critical Rules

1. **Always include text content** when using `structuredContent`
2. **Never stringify JSON** in text field
3. **Use concise summary for large data** (avoid duplication)
4. **Define outputSchema** for structured data
5. **Use appropriate content type** for binary data (image/audio)
6. **Provide actionable error messages** with context

## Content Types Quick Reference

| Type | Use When | Example |
|------|----------|---------|
| `text` | Simple messages | "File saved" |
| `structuredContent` | JSON objects/arrays | Search results, data |
| `resource` | File with context | Source code, config |
| `resource_link` | External reference | URL, file path |
| `image` | Visual data | Charts, screenshots |
| `audio` | Sound data | Speech, music |

## Error Handling

**Actionable Errors** (AI can fix):
```json
{
  "content": [{"type": "text", "text": "Invalid date: use YYYY-MM-DD"}],
  "isError": true
}
```

**Protocol Errors** (structural):
```json
{
  "error": {
    "code": -32602,
    "message": "Unknown tool: tool_name"
  }
}
```

## Checklist Before Shipping

- [ ] Chosen correct content type
- [ ] Text fallback for structured content
- [ ] **Concise summary for large data** (no duplication)
- [ ] Output schema defined (if structured)
- [ ] Error messages are actionable
- [ ] Tested with multiple clients
- [ ] No JSON in text field
- [ ] Annotations added (if needed)

## Performance Tips

**For Large Data (>100 entries or >10 KB):**

```rust
// ❌ DON'T - Duplicates data
CallToolResult::structured(data)

// ✅ DO - Concise summary
CallToolResult {
    content: vec![Content::text("Found 1000 items")],
    structured_content: Some(data),
    is_error: Some(false),
    meta: None,
}
```

**Impact:** 50% smaller responses, faster transmission

## See Also

- [Complete Guide](tool-output-formats.md) - Full documentation
- [Adding Tools](../guides/adding-tools.md) - Tool implementation guide
- [MCP Specification](https://modelcontextprotocol.io/specification/2025-06-18/server/tools)
