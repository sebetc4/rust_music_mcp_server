# Filesystem Tools

This directory contains documentation for filesystem manipulation tools in the Music MCP Server.

## Available Tools

### Core Operations
- **[fs_list_dir](fs_list_dir.md)** - List directory contents with recursive support
- **[fs_rename](fs_rename.md)** - Rename files and directories with dry-run support
- **[fs_delete](fs_delete.md)** - Delete files and directories with safety checks

## Quick Comparison

| Tool | Purpose | Recursive | Dry Run | Reversible | Output Format |
|------|---------|-----------|---------|------------|---------------|
| [fs_list_dir](fs_list_dir.md) | Read directory contents | ✅ Yes | N/A | N/A | JSON |
| [fs_rename](fs_rename.md) | Rename files/directories | ❌ No | ✅ Yes | ✅ Yes | Text |
| [fs_delete](fs_delete.md) | Delete files/directories | ✅ Yes | ❌ No | ❌ No | JSON |

## Common Use Cases

### Music Library Management

1. **Scan Artist Directory**
   ```json
   // Use fs_list_dir with depth=2
   {
     "path": "/music/Artist Name",
     "recursive_depth": 2
   }
   ```

2. **Rename Album**
   ```json
   // Use fs_rename with dry-run first
   {
     "old_path": "/music/Artist/Album",
     "new_path": "/music/Artist/Album (Deluxe)",
     "dry_run": true
   }
   ```

3. **Find All Audio Files**
   ```json
   // Use fs_list_dir with full recursion
   {
     "path": "/music",
     "recursive_depth": -1,
     "detailed": true
   }
   ```

4. **Delete Duplicate Files**
   ```json
   // Use fs_delete after identifying duplicates
   {
     "path": "/music/Artist/duplicate.mp3"
   }
   ```

5. **Clean Up Empty Directories**
   ```json
   // Use fs_delete on empty directories
   {
     "path": "/music/empty_folder",
     "recursive": false
   }
   ```

## Security & Safety

All filesystem tools implement:

- ✅ **Path validation** against configured root directory
- ✅ **Security constraints** prevent path traversal attacks
- ✅ **Error handling** with graceful degradation
- ✅ **Detailed logging** for audit trails

See [Path Security](../../reference/path-security.md) for implementation details.

## Tool Integration

These tools work seamlessly with other MCP tools:

```
fs_list_dir → discover files
     ↓
read_metadata → analyze tags
     ↓
mb_identify_record → match to MusicBrainz
     ↓
write_metadata → update tags
     ↓
fs_rename → organize files
     ↓
fs_delete → remove duplicates
```

## Configuration

Set root directory constraints via environment variables:

```bash
export MCP_ROOT_PATH="/music"
export MCP_ALLOW_SYMLINKS=true
```

See [Configuration Guide](../../guides/configuration.md) for details.

## Best Practices

### 1. Always Use Dry Run First
```json
// Step 1: Test with dry_run
{"old_path": "...", "new_path": "...", "dry_run": true}

// Step 2: Execute if safe
{"old_path": "...", "new_path": "...", "dry_run": false}
```

### 2. Limit Recursive Depth
```json
// Bad: Unlimited depth on large library
{"path": "/music", "recursive_depth": -1}

// Good: Limited depth for initial scan
{"path": "/music", "recursive_depth": 2}
```

### 3. Use Appropriate Tools
- ❌ Don't use `fs_rename` for batch operations (no wildcard support)
- ✅ Use `fs_list_dir` to discover files, then rename individually
- ✅ Check existence with `fs_list_dir` before renaming

### 4. Verify Before Deleting
```json
// Bad: Delete without checking
{"path": "/music/unknown", "recursive": true}

// Good: List first, then delete
{"tool": "fs_list_dir", "path": "/music/unknown"}
// ... review output ...
{"tool": "fs_delete", "path": "/music/unknown", "recursive": true}
```

## Related Documentation

- [Adding New Tools](../../guides/adding-tools.md) - Create custom filesystem tools
- [Path Security](../../reference/path-security.md) - Security implementation
- [Error Handling](../../reference/error-handling.md) - Error patterns
- [Testing Guide](../../guides/testing.md) - Testing strategies

## Tool-Specific Documentation

- [fs_list_dir.md](fs_list_dir.md) - Detailed `fs_list_dir` documentation
- [fs_rename.md](fs_rename.md) - Detailed `fs_rename` documentation
- [fs_delete.md](fs_delete.md) - Detailed `fs_delete` documentation
