# fs_rename

Rename or move files and directories from one path to another with safety checks.

## Overview

The `fs_rename` tool provides safe file and directory renaming/moving capabilities with:

- 🔒 Path security validation (both source and destination)
- ✅ Overwrite protection (optional)
- 📝 Clear success/error messages
- 🔍 Parent directory validation for new files

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `from` | string | ✅ Yes | - | Source path (file or directory to rename/move) |
| `to` | string | ✅ Yes | - | Destination path (new name or location) |
| `overwrite` | boolean | ❌ No | `false` | Overwrite destination if it already exists |

## Output Format

Returns structured JSON with both human-readable summary and machine-parseable data:

### Success Response

**Structure:**
```json
{
  "content": [{
    "type": "text",
    "text": "Successfully renamed file from '/old/path' to '/new/path'"
  }],
  "isError": false,
  "structuredContent": {
    "from": "/old/path",
    "to": "/new/path",
    "item_type": "file",
    "operation": "renamed",
    "success": true
  }
}
```

### Fields in structuredContent

| Field | Type | Description | Possible Values |
|-------|------|-------------|-----------------|
| `from` | string | Source path (original location) | Absolute path |
| `to` | string | Destination path (new location) | Absolute path |
| `item_type` | string | Type of item renamed | `"file"`, `"directory"`, `"item"` |
| `operation` | string | Operation performed | `"renamed"` (same dir), `"moved"` (different dir) |
| `success` | boolean | Whether operation succeeded | `true`, `false` |
| `overwritten` | boolean? | Whether existing file was overwritten | Optional, `true` if applicable |

### MCP Output Format

This tool follows MCP best practices:

1. **Text Summary** (human-readable): Concise description of what happened
2. **Structured Content** (machine-readable): Complete data for AI agents to parse

For more information on MCP output formats, see [Tool Output Formats Guide](../../reference/tool-output-formats.md).

### Error Response

```json
{
  "content": [{
    "type": "text",
    "text": "Error: [description of what went wrong]"
  }],
  "isError": true
}
```

## Examples

### Example 1: Simple Rename (Same Directory)

**Request:**
```json
{
  "from": "/music/Artist/Album/01-trak.mp3",
  "to": "/music/Artist/Album/01-track.mp3"
}
```

**Response:**
```json
{
  "content": [{
    "type": "text",
    "text": "Successfully renamed file from '/music/Artist/Album/01-trak.mp3' to '/music/Artist/Album/01-track.mp3'"
  }],
  "isError": false,
  "structuredContent": {
    "from": "/music/Artist/Album/01-trak.mp3",
    "to": "/music/Artist/Album/01-track.mp3",
    "item_type": "file",
    "operation": "renamed",
    "success": true
  }
}
```

### Example 2: Rename Directory

**Request:**
```json
{
  "from": "/music/Artist/Album",
  "to": "/music/Artist/Album (Deluxe Edition)"
}
```

**Response:**
```json
{
  "content": [{
    "type": "text",
    "text": "Successfully renamed directory from '/music/Artist/Album' to '/music/Artist/Album (Deluxe Edition)'"
  }],
  "isError": false,
  "structuredContent": {
    "from": "/music/Artist/Album",
    "to": "/music/Artist/Album (Deluxe Edition)",
    "item_type": "directory",
    "operation": "renamed",
    "success": true
  }
}
```

### Example 3: Move to Different Directory

**Request:**
```json
{
  "from": "/music/Unsorted/track.mp3",
  "to": "/music/Artist Name/Album/01-track.mp3"
}
```

**Response:**
```json
{
  "content": [{
    "type": "text",
    "text": "Successfully moved file from '/music/Unsorted/track.mp3' to '/music/Artist Name/Album/01-track.mp3'"
  }],
  "isError": false,
  "structuredContent": {
    "from": "/music/Unsorted/track.mp3",
    "to": "/music/Artist Name/Album/01-track.mp3",
    "item_type": "file",
    "operation": "moved",
    "success": true
  }
}
```

**Note:** The `operation` field is `"moved"` because the parent directories are different.

### Example 4: Overwrite Existing File

**Request:**
```json
{
  "from": "/music/New/track.mp3",
  "to": "/music/Artist/Album/01-track.mp3",
  "overwrite": true
}
```

**Response:**
```json
{
  "content": [{
    "type": "text",
    "text": "Successfully moved file from '/music/New/track.mp3' to '/music/Artist/Album/01-track.mp3'"
  }],
  "isError": false,
  "structuredContent": {
    "from": "/music/New/track.mp3",
    "to": "/music/Artist/Album/01-track.mp3",
    "item_type": "file",
    "operation": "moved",
    "success": true,
    "overwritten": true
  }
}
```

**Note:** The `overwritten` field indicates that an existing file was replaced.

### Example 5: Rename with Special Characters

**Request:**
```json
{
  "from": "/music/Artist/Album/track.mp3",
  "to": "/music/Artist/Album/01 - Track: The Beginning.mp3"
}
```

**Response:**
```json
{
  "content": [{
    "type": "text",
    "text": "Successfully renamed file from '/music/Artist/Album/track.mp3' to '/music/Artist/Album/01 - Track: The Beginning.mp3'"
  }],
  "isError": false,
  "structuredContent": {
    "from": "/music/Artist/Album/track.mp3",
    "to": "/music/Artist/Album/01 - Track: The Beginning.mp3",
    "item_type": "file",
    "operation": "renamed",
    "success": true
  }
}
```

## Using Structured Content (AI Agents)

AI agents can access both the human-readable summary and structured data:

### Accessing the Summary

```javascript
const summary = result.content[0].text;
// "Successfully renamed file from 'old.mp3' to 'new.mp3'"
```

### Parsing Structured Data

```javascript
const data = result.structuredContent;

// Check operation type
if (data.operation === "moved") {
  console.log(`File moved from ${data.from} to ${data.to}`);
} else {
  console.log(`File renamed in same directory`);
}

// Check if file was overwritten
if (data.overwritten) {
  console.log("⚠️ Warning: Existing file was replaced");
}

// Verify success
if (data.success && data.item_type === "file") {
  // File operation completed successfully
}
```

### Type Detection

```javascript
switch (data.item_type) {
  case "file":
    // Handle file rename
    break;
  case "directory":
    // Handle directory rename (affects all contained files)
    break;
  case "item":
    // Handle other types (symlinks, etc.)
    break;
}
```

## Error Handling

### Error: Source Doesn't Exist

**Request:**
```json
{
  "from": "/music/nonexistent.mp3",
  "to": "/music/renamed.mp3"
}
```

**Response:**
```json
{
  "isError": true,
  "message": "Source path does not exist: /music/nonexistent.mp3"
}
```

### Error: Destination Already Exists

**Request:**
```json
{
  "from": "/music/track1.mp3",
  "to": "/music/track2.mp3",
  "overwrite": false
}
```

**Response:**
```json
{
  "isError": true,
  "message": "Destination path already exists: /music/track2.mp3. Use overwrite=true to replace."
}
```

### Error: Path Security Violation

**Request:**
```json
{
  "from": "/music/track.mp3",
  "to": "/etc/passwd"
}
```

**Response:**
```json
{
  "isError": true,
  "message": "Destination path security validation failed: path outside root directory"
}
```

### Error: Parent Directory Doesn't Exist

**Request:**
```json
{
  "from": "/music/track.mp3",
  "to": "/music/nonexistent/subdir/track.mp3"
}
```

**Response:**
```json
{
  "isError": true,
  "message": "Destination parent directory does not exist: /music/nonexistent/subdir"
}
```

### Error: Permission Denied

**Request:**
```json
{
  "from": "/music/track.mp3",
  "to": "/music/restricted/track.mp3"
}
```

**Response:**
```json
{
  "isError": true,
  "message": "Permission denied: cannot write to /music/restricted/"
}
```

## Use Cases

### 1. Correct Track Names

**Goal:** Fix typos in track filenames

```json
{
  "from": "/music/Artist/Album/01-Trcak Name.mp3",
  "to": "/music/Artist/Album/01-Track Name.mp3"
}
```

**Result:** Fixed filename without moving file

### 2. Add Track Numbers

**Goal:** Add track numbers to filenames

```json
{
  "from": "/music/Artist/Album/Song Title.mp3",
  "to": "/music/Artist/Album/01-Song Title.mp3"
}
```

**Result:** Track number prepended to filename

### 3. Organize Albums

**Goal:** Rename album directory with additional info

```json
{
  "from": "/music/Artist/Album",
  "to": "/music/Artist/Album (2024) [Remaster]"
}
```

**Result:** Album directory renamed with year and edition info

### 4. Move to Proper Location

**Goal:** Move misplaced track to correct album

```json
{
  "from": "/music/Unsorted/bonus_track.mp3",
  "to": "/music/Artist/Album (Deluxe)/Disc 2/01-Bonus Track.mp3"
}
```

**Result:** Track moved and renamed to proper location

### 5. Replace Corrupted File

**Goal:** Replace a corrupted file with a good copy

```json
{
  "from": "/music/Backup/track.mp3",
  "to": "/music/Artist/Album/01-track.mp3",
  "overwrite": true
}
```

**Result:** Old corrupted file replaced with backup

## Best Practices

### 1. Always Check Source First

Use `fs_list_dir` to verify the source path exists:

```json
// Step 1: List directory
{"path": "/music/Artist/Album", "recursive_depth": 0}

// Step 2: Rename based on listing
{"from": "/music/Artist/Album/trak.mp3", "to": "/music/Artist/Album/track.mp3"}
```

### 2. Be Careful with Overwrite

**Don't:**
```json
{"from": "...", "to": "...", "overwrite": true}  // Dangerous!
```

**Do:**
```json
// Step 1: Check if destination exists
{"path": "/music/Artist/Album", "recursive_depth": 0}

// Step 2: Decide based on result
{"from": "...", "to": "...", "overwrite": false}
```

### 3. Validate Destination Parent

Ensure destination parent directory exists:

```json
// Step 1: List parent directory
{"path": "/music/Artist/Album", "recursive_depth": 0}

// Step 2: Rename into existing directory
{"from": "...", "to": "/music/Artist/Album/new-name.mp3"}
```

### 4. Handle Batch Operations Carefully

For multiple renames, process one at a time:

```javascript
// BAD: Parallel renames might conflict
Promise.all([rename1, rename2, rename3])

// GOOD: Sequential renames
await rename1
await rename2
await rename3
```

### 5. Use Descriptive Names

Include track numbers and metadata in filenames:

```
❌ Bad:  track.mp3
✅ Good: 01-Track Title.mp3
✅ Good: 01 - Artist - Track Title.mp3
```

## Security

### Path Validation

Both source and destination paths are validated:

**Source validation:**
```rust
validate_path(&params.from, config)?
```

**Destination validation:**
- If destination exists: validate full path
- If destination doesn't exist: validate parent directory

**Prevents:**
- ✅ Moving files outside root directory
- ✅ Path traversal attacks (`../../../etc/`)
- ✅ Symlink exploits

### Overwrite Protection

Default behavior (`overwrite: false`) prevents accidental data loss:

```json
{
  "from": "/music/new.mp3",
  "to": "/music/existing.mp3",
  "overwrite": false  // Will fail if existing.mp3 exists
}
```

**Must explicitly enable overwrite:**
```json
{
  "overwrite": true  // Explicitly allow overwriting
}
```

## Atomic Operations

The tool uses filesystem `rename()` operation which is atomic on most systems:

- ✅ Either succeeds completely or fails completely
- ✅ No partial state (file half-moved)
- ✅ Thread-safe (concurrent calls don't conflict)

**Note:** Atomicity only guaranteed when source and destination are on the same filesystem.

## Limitations

### 1. No Wildcard Support

Cannot rename multiple files at once:

```json
// ❌ NOT SUPPORTED
{
  "from": "/music/Album/*.mp3",
  "to": "/music/Album/renamed-*.mp3"
}
```

**Workaround:** Use `fs_list_dir` to discover files, then rename individually.

### 2. No Directory Merging

Cannot merge directories:

```json
// ❌ FAILS if /dest/ exists with files
{
  "from": "/music/Album1/",
  "to": "/music/Album2/"
}
```

**Workaround:** Move files individually.

### 3. Cross-Filesystem Moves

Moving across filesystems may not be atomic:

```json
// May not be atomic
{
  "from": "/music/track.mp3",      // filesystem 1
  "to": "/backup/track.mp3"        // filesystem 2
}
```

**Note:** Most music libraries are on single filesystem.

### 4. No Undo

Operations are permanent:

```json
// ❌ No automatic undo
{"from": "/music/important.mp3", "to": "/music/renamed.mp3"}
```

**Workaround:** Rename back manually if needed.

## Performance

| Operation | Typical Time | Notes |
|-----------|--------------|-------|
| Rename (same dir) | ~1ms | Just updates directory entry |
| Move (same filesystem) | ~1ms | Updates directory entry |
| Move (cross filesystem) | Depends on size | Actual file copy + delete |

**Fast operations:** Same filesystem, same directory
**Slow operations:** Cross-filesystem moves of large files

## Implementation Details

**Source:** [`src/domains/tools/definitions/fs/rename.rs`](../../../src/domains/tools/definitions/fs/rename.rs)

**Key Features:**
- Uses Rust `std::fs::rename()`
- Validates both source and destination paths
- Checks parent directory existence
- Thread-safe (can be called concurrently)
- Atomic on same filesystem

**Algorithm:**
1. Validate source path exists and is within root
2. Validate destination path (or parent) is within root
3. Check if destination exists (if `overwrite: false`)
4. Execute filesystem rename operation
5. Return success/error message

## Testing

Comprehensive test coverage:

```bash
cargo test --features all rename --lib
```

**Test Coverage:**
- ✅ Basic rename
- ✅ Move to different directory
- ✅ Overwrite protection
- ✅ Path security validation
- ✅ Nonexistent source handling
- ✅ Existing destination handling
- ✅ HTTP handlers

## Troubleshooting

### Issue: "Source path does not exist"

**Symptom:** Error even though file seems to exist

**Possible Causes:**
1. Typo in source path
2. File was moved/deleted
3. Path is outside security root

**Solution:** Use `fs_list_dir` to confirm exact path

### Issue: "Destination already exists"

**Symptom:** Rename fails even though you want to replace

**Solution:** Add `"overwrite": true` to request

### Issue: "Parent directory does not exist"

**Symptom:** Cannot create file in new location

**Solution:** Create parent directory first (currently not supported by tools)

### Issue: "Permission denied"

**Symptom:** Cannot rename/move file

**Possible Causes:**
1. No write permission on source directory
2. No write permission on destination directory
3. File is open/locked by another process

**Solution:** Check filesystem permissions

## Related Tools

- **[fs_list_dir](fs_list_dir.md)** - Discover files to rename
- **[read_metadata](../metadata/read_metadata.md)** - Read metadata for proper naming
- **[write_metadata](../metadata/write_metadata.md)** - Update metadata after organizing

## Workflow Example

Complete workflow for organizing music:

```json
// 1. List unsorted files
{"path": "/music/Unsorted", "recursive_depth": 1}

// 2. Read metadata to get proper names
{"path": "/music/Unsorted/track.mp3"}

// 3. Rename with proper metadata
{
  "from": "/music/Unsorted/track.mp3",
  "to": "/music/Artist Name/Album Title/01-Track Title.mp3"
}

// 4. Verify new location
{"path": "/music/Artist Name/Album Title", "recursive_depth": 0}
```

## Related Documentation

- [Path Security](../../reference/path-security.md) - Security implementation
- [Configuration Guide](../../guides/configuration.md) - Environment setup
- [Error Handling](../../reference/error-handling.md) - Error patterns
