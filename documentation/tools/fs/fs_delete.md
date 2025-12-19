# fs_delete

Delete files and directories with safety checks and recursive support. Returns structured JSON for AI agents.

## Overview

The `fs_delete` tool provides safe deletion capabilities with built-in safeguards, making it ideal for:

- 🗑️ Removing unwanted files from music libraries
- 🧹 Cleaning up temporary or duplicate files
- 📁 Deleting empty or non-empty directories
- 🔒 Safe deletion with path validation

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path` | string | ✅ Yes | - | Path to the file or directory to delete |
| `recursive` | boolean | ❌ No | `false` | Recursively delete directories and their contents |

### Recursive Flag Behavior

| Scenario | `recursive: false` | `recursive: true` |
|----------|-------------------|-------------------|
| **Single file** | ✅ Deletes file | ✅ Deletes file |
| **Empty directory** | ✅ Deletes directory | ✅ Deletes directory |
| **Non-empty directory** | ❌ Error: "Directory is not empty" | ✅ Deletes directory and all contents |

## Output Format

Returns structured JSON with deletion details:

```json
{
  "path": "/path/to/deleted/item",
  "item_type": "file",        // "file", "directory", or "item"
  "success": true,
  "recursive": true            // Only present if recursive deletion was used
}
```

### Output Fields

- **`path`**: Path that was deleted (echoes request)
- **`item_type`**: Type of item deleted
  - `"file"`: Regular file
  - `"directory"`: Directory (empty or recursive)
  - `"item"`: Unknown type (rare)
- **`success`**: Always `true` in successful responses
- **`recursive`**: Present only when `recursive: true` was used for a directory

### MCP Output Format

This tool follows MCP best practices by returning data in two forms:

1. **Text Summary** (human-readable):
   - Success: "Successfully deleted file 'example.mp3'"
   - Recursive: "Successfully deleted directory 'folder' and all its contents"
2. **Structured Content** (machine-readable): The JSON structure shown above

AI agents can directly parse the `structuredContent` field for programmatic access to deletion results.

For more information on MCP output formats, see [Tool Output Formats Guide](../../reference/tool-output-formats.md).

## Examples

### Delete a Single File

**Request:**
```json
{
  "path": "/music/duplicates/song.mp3",
  "recursive": false
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Successfully deleted file '/music/duplicates/song.mp3'"
    }
  ],
  "structuredContent": {
    "path": "/music/duplicates/song.mp3",
    "item_type": "file",
    "success": true
  },
  "isError": false
}
```

### Delete an Empty Directory

**Request:**
```json
{
  "path": "/music/empty_folder",
  "recursive": false
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Successfully deleted directory '/music/empty_folder'"
    }
  ],
  "structuredContent": {
    "path": "/music/empty_folder",
    "item_type": "directory",
    "success": true
  },
  "isError": false
}
```

### Attempt to Delete Non-Empty Directory (Without Recursive)

**Request:**
```json
{
  "path": "/music/artist/album",
  "recursive": false
}
```

**Error Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Directory is not empty: /music/artist/album. Use recursive=true to delete it and its contents."
    }
  ],
  "isError": true
}
```

### Delete Non-Empty Directory (With Recursive)

**Request:**
```json
{
  "path": "/music/artist/album",
  "recursive": true
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Successfully deleted directory '/music/artist/album' and all its contents"
    }
  ],
  "structuredContent": {
    "path": "/music/artist/album",
    "item_type": "directory",
    "success": true,
    "recursive": true
  },
  "isError": false
}
```

## Error Handling

The tool provides helpful error messages for common scenarios:

### Path Does Not Exist
```json
{
  "content": [{"type": "text", "text": "Path does not exist: /nonexistent/file.mp3"}],
  "isError": true
}
```

### Permission Denied
```json
{
  "content": [{"type": "text", "text": "Permission denied: Cannot delete '/protected/file.mp3'"}],
  "isError": true
}
```

### Path Security Violation
```json
{
  "content": [{"type": "text", "text": "Path security validation failed: Path is outside allowed root directory"}],
  "isError": true
}
```

### Directory Not Empty (Without Recursive)
```json
{
  "content": [{"type": "text", "text": "Directory is not empty: /path/to/dir. Use recursive=true to delete it and its contents."}],
  "isError": true
}
```

## Security & Safety

### Path Validation

All deletion operations are subject to strict path security validation:

- ✅ **Root directory enforcement** - Cannot delete outside configured root
- ✅ **Path traversal prevention** - Blocks `..` and symlink attacks
- ✅ **Explicit paths only** - No wildcards or pattern matching
- ✅ **Pre-deletion checks** - Validates path exists and type before deleting

See [Path Security Reference](../../reference/path-security.md) for implementation details.

### Safety Checklist

Before using `fs_delete`, consider:

1. **Backup important data** - Deletion is permanent
2. **Use [fs_list_dir](fs_list_dir.md) first** - Verify contents before recursive delete
3. **Start with single files** - Test behavior before batch operations
4. **Check permissions** - Ensure proper file access rights
5. **Verify paths** - Double-check path correctness

## Use Cases

### Remove Duplicate Files

```json
// Step 1: Identify duplicate
{"tool": "mb_identify_record", "path": "/music/song_copy.mp3"}

// Step 2: Delete duplicate
{"tool": "fs_delete", "path": "/music/song_copy.mp3", "recursive": false}
```

### Clean Up Failed Import

```json
// Delete entire failed import directory
{
  "path": "/music/import_2024_failed",
  "recursive": true
}
```

### Remove Empty Directories

```json
// Step 1: List directory to confirm empty
{"tool": "fs_list_dir", "path": "/music/old_artist"}

// Step 2: Delete if empty
{"tool": "fs_delete", "path": "/music/old_artist", "recursive": false}
```

## Best Practices

### 1. Always Verify Before Deleting

```json
// Bad: Delete without checking
{"path": "/music/unknown_folder", "recursive": true}

// Good: List contents first
{"tool": "fs_list_dir", "path": "/music/unknown_folder"}
// ... review output ...
{"tool": "fs_delete", "path": "/music/unknown_folder", "recursive": true}
```

### 2. Use Recursive Flag Carefully

```json
// Bad: Always use recursive
{"path": "/any/path", "recursive": true}

// Good: Use recursive only when needed
{"path": "/empty/dir", "recursive": false}  // For empty directories
{"path": "/nonempty/dir", "recursive": true}  // Only when necessary
```

### 3. Handle Errors Gracefully

```typescript
const result = await mcpClient.callTool("fs_delete", {
  path: targetPath,
  recursive: false
});

if (result.isError) {
  if (result.content[0].text.includes("not empty")) {
    // Ask user before recursive delete
    const confirm = await promptUser("Directory not empty. Delete all contents?");
    if (confirm) {
      await mcpClient.callTool("fs_delete", {path: targetPath, recursive: true});
    }
  }
}
```

## Comparison with Other Tools

| Tool | Purpose | Recursive | Reversible | Validation |
|------|---------|-----------|------------|------------|
| [fs_delete](fs_delete.md) | **Delete** files/dirs | ✅ Yes | ❌ No | ✅ Path security |
| [fs_rename](fs_rename.md) | **Move/rename** files | ❌ No | ✅ Can undo | ✅ Path security + dry-run |
| [fs_list_dir](fs_list_dir.md) | **Read** directory | ✅ Yes | N/A | ✅ Path security |

**Key Difference**: `fs_delete` is **irreversible** - deleted files cannot be recovered through this tool.

## Limitations

- ❌ **No wildcards** - Must specify exact path (use [fs_list_dir](fs_list_dir.md) to discover files)
- ❌ **No undo** - Deletion is permanent
- ❌ **No trash/recycle bin** - Files are immediately removed from filesystem
- ❌ **No batch operations** - One path per call
- ✅ **Root directory constraint** - Cannot delete outside configured root

## Configuration

The delete tool respects the global path security configuration:

```bash
# Set root directory (deletions restricted to this path)
export MCP_ROOT_PATH="/music"

# Allow symlink following (use with caution)
export MCP_ALLOW_SYMLINKS=false  # Recommended for safety
```

See [Configuration Guide](../../guides/configuration.md) for details.

## Related Documentation

- [fs_list_dir](fs_list_dir.md) - List directory contents before deleting
- [fs_rename](fs_rename.md) - Move files instead of deleting
- [Path Security](../../reference/path-security.md) - Security implementation details
- [Tool Output Formats](../../reference/tool-output-formats.md) - MCP output format guide
- [Error Handling](../../reference/error-handling.md) - Error patterns and recovery

## Implementation Details

**Source Code**: [src/domains/tools/definitions/fs/delete.rs](../../../src/domains/tools/definitions/fs/delete.rs)

**Key Features**:
- Path security validation before all operations
- Type detection (file vs directory) before deletion
- Empty directory check when `recursive: false`
- Detailed error messages with actionable guidance
- Structured JSON output for AI agents
- Full test coverage (see tests in source file)

**Transport Support**:
- ✅ STDIO (default)
- ✅ TCP
- ✅ HTTP
