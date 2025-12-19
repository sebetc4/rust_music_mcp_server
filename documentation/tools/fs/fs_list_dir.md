# fs_list_dir

List files and directories in a given path with optional recursive traversal. Returns JSON format optimized for AI agents.

## Overview

The `fs_list_dir` tool provides comprehensive directory listing capabilities with configurable recursion depth, making it ideal for:

- 🎵 Scanning music library structures
- 📊 Analyzing directory organization
- 🔍 Discovering files for batch processing
- 📈 Storage usage analysis (with `detailed` mode)

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path` | string | ✅ Yes | - | Directory path to list |
| `include_hidden` | boolean | ❌ No | `false` | Include hidden files (starting with '.') |
| `detailed` | boolean | ❌ No | `false` | Include file sizes in bytes (only for files) |
| `recursive_depth` | integer | ❌ No | `0` | Recursion depth (see below) |

### Recursive Depth Values

| Value | Behavior | Use Case |
|-------|----------|----------|
| `0` | **Non-recursive** (default)<br>Lists only direct children | Quick directory overview |
| `1` | **One level deep**<br>Includes files in immediate subdirectories | Artist → Albums listing |
| `2` | **Two levels deep**<br>Includes files in subdirectories and their subdirectories | Artist → Albums → Tracks |
| `3+` | **N levels deep** | Deep directory structures |
| `-1` | **Unlimited** (capped at 10)<br>Traverses entire tree | Complete library scan |

## Output Format

Returns structured JSON in **hierarchical format** for natural tree representation:

```json
{
  "path": "/requested/path",
  "entries": [
    {
      "name": "filename.mp3",
      "type": "file",
      "size": 12345  // Optional: only in detailed mode for files
    },
    {
      "name": "subdirectory",
      "type": "directory",
      "children": [  // Nested structure for subdirectories
        {
          "name": "nested-file.mp3",
          "type": "file"
        }
      ]
    }
  ],
  "dir_count": 5,
  "file_count": 10,
  "warnings": []  // Optional: only present if issues occurred
}
```

### Entry Fields

- **`name`**: Just the filename or directory name (not full path)
- **`type`**: One of `"file"`, `"directory"`, or `"symlink"`
- **`size`**: File size in bytes (only present if `detailed: true` and entry is a file)
- **`children`**: Array of nested entries (only for directories when recursing, omitted if empty)

### Result Fields

- **`path`**: The directory that was listed (echoes request)
- **`entries`**: Array of file/directory entries (sorted alphabetically)
- **`dir_count`**: Total number of directories found
- **`file_count`**: Total number of files found
- **`warnings`**: Array of warning messages (omitted if empty)

### MCP Output Format

This tool follows MCP best practices by returning data in two forms:

1. **Text Summary** (human-readable): e.g., "Found 2 directories and 3 files in '/music/Artist'"
2. **Structured Content** (machine-readable): The JSON structure shown above

AI agents can directly parse the `structuredContent` field without needing to parse JSON from text. This avoids double-parsing and provides type-safe access to the data.

For more information on MCP output formats, see [Tool Output Formats Guide](../../reference/tool-output-formats.md).

## Using Structured Content (AI Agents)

AI agents can efficiently navigate the hierarchical file structure:

### Accessing the Summary

```javascript
const summary = result.content[0].text;
// "Found 16 directories and 1000 files in '/music'"
```

### Traversing the Hierarchy

```javascript
const data = result.structuredContent;

// Get total counts
console.log(`Total: ${data.dir_count} dirs, ${data.file_count} files`);

// Recursive function to traverse tree
function traverse(entries, depth = 0) {
  for (const entry of entries) {
    const indent = "  ".repeat(depth);
    console.log(`${indent}${entry.name} (${entry.type})`);

    // Recurse into directories
    if (entry.children && entry.children.length > 0) {
      traverse(entry.children, depth + 1);
    }
  }
}

traverse(data.entries);
```

### Filtering by Type

```javascript
// Find all directories
const directories = data.entries.filter(e => e.type === "directory");

// Find all MP3 files recursively
function findFiles(entries, extension) {
  let files = [];
  for (const entry of entries) {
    if (entry.type === "file" && entry.name.endsWith(extension)) {
      files.push(entry);
    }
    if (entry.children) {
      files.push(...findFiles(entry.children, extension));
    }
  }
  return files;
}

const mp3Files = findFiles(data.entries, ".mp3");
```

### Building Full Paths

```javascript
// Reconstruct full paths from hierarchy
function buildPaths(entries, basePath = "") {
  const paths = [];
  for (const entry of entries) {
    const fullPath = basePath + "/" + entry.name;
    paths.push({
      path: fullPath,
      type: entry.type,
      size: entry.size
    });

    if (entry.children) {
      paths.push(...buildPaths(entry.children, fullPath));
    }
  }
  return paths;
}

const allPaths = buildPaths(data.entries, data.path);
```

### Checking for Warnings

```javascript
// Check if results were truncated
if (data.warnings && data.warnings.length > 0) {
  console.log("⚠️ Warnings:");
  data.warnings.forEach(w => console.log(`  - ${w}`));

  // Typical warnings:
  // - "Results truncated: exceeded maximum of 1000 entries"
  // - "Depth limited to 10 levels"
  // - "Could not read directory '/restricted': Permission denied"
}
```

## Safety Limits

The tool enforces strict safety limits to prevent resource exhaustion:

| Limit | Value | Behavior |
|-------|-------|----------|
| **Max Depth** | 10 levels | Automatically caps even with `-1` (unlimited) |
| **Max Entries** | 1000 items | Stops traversal and adds warning |

When limits are reached, the tool:
- ✅ Returns partial results (what was found before limit)
- ✅ Adds descriptive warnings to the output
- ✅ Continues to be responsive (doesn't hang)

## Examples

### Example 1: Basic Non-Recursive Listing

**Request:**
```json
{
  "path": "/music/Artist/Album"
}
```

**Response:**
```json
{
  "path": "/music/Artist/Album",
  "entries": [
    {
      "name": "01-track.mp3",
      "type": "file"
    },
    {
      "name": "02-track.mp3",
      "type": "file"
    },
    {
      "name": "cover.jpg",
      "type": "file"
    }
  ],
  "dir_count": 0,
  "file_count": 3
}
```

### Example 2: Artist Albums (Depth 1)

**Request:**
```json
{
  "path": "/music/Artist Name",
  "recursive_depth": 1
}
```

**Response:**
```json
{
  "path": "/music/Artist Name",
  "entries": [
    {
      "name": "Album 1",
      "type": "directory",
      "children": [
        {
          "name": "01-track.mp3",
          "type": "file"
        },
        {
          "name": "02-track.mp3",
          "type": "file"
        }
      ]
    },
    {
      "name": "Album 2",
      "type": "directory",
      "children": [
        {
          "name": "01-single.mp3",
          "type": "file"
        }
      ]
    }
  ],
  "dir_count": 2,
  "file_count": 3
}
```

### Example 3: Full Structure with Sizes (Depth 2)

**Request:**
```json
{
  "path": "/music/Artist",
  "recursive_depth": 2,
  "detailed": true
}
```

**Response:**
```json
{
  "path": "/music/Artist",
  "entries": [
    {
      "name": "Album 1",
      "type": "directory",
      "children": [
        {
          "name": "01-track.mp3",
          "type": "file",
          "size": 3456789
        },
        {
          "name": "02-track.mp3",
          "type": "file",
          "size": 4123456
        }
      ]
    },
    {
      "name": "Album 2",
      "type": "directory",
      "children": [
        {
          "name": "Bonus Disc",
          "type": "directory",
          "children": [
            {
              "name": "01-bonus.mp3",
              "type": "file",
              "size": 2987654
            }
          ]
        }
      ]
    }
  ],
  "dir_count": 3,
  "file_count": 3
}
```

### Example 4: Complete Library Scan (Unlimited)

**Request:**
```json
{
  "path": "/music",
  "recursive_depth": -1,
  "include_hidden": false
}
```

**Response (with warnings):**
```json
{
  "path": "/music",
  "entries": [
    "... (first 1000 entries) ..."
  ],
  "dir_count": 300,
  "file_count": 700,
  "warnings": [
    "Results truncated: exceeded maximum of 1000 entries. Consider reducing recursive_depth.",
    "Depth limited to 10 levels for safety (requested unlimited)."
  ]
}
```

### Example 5: Including Hidden Files

**Request:**
```json
{
  "path": "/music/Artist/Album",
  "include_hidden": true
}
```

**Response:**
```json
{
  "path": "/music/Artist/Album",
  "entries": [
    {
      "name": ".DS_Store",
      "type": "file"
    },
    {
      "name": "01-track.mp3",
      "type": "file"
    }
  ],
  "dir_count": 0,
  "file_count": 2
}
```

## Error Handling

The tool handles errors gracefully and continues traversal when possible:

### Permission Errors

**Scenario:** Subdirectory is not readable

**Response:**
```json
{
  "path": "/music",
  "entries": [
    {
      "name": "accessible_album",
      "type": "directory"
    }
  ],
  "dir_count": 1,
  "file_count": 0,
  "warnings": [
    "Could not read directory '/music/restricted': Permission denied"
  ]
}
```

### Symlink Loops (Unix)

**Scenario:** Circular symlink detected

**Response:**
```json
{
  "warnings": [
    "Skipped '/music/link_to_parent': symlink loop detected"
  ]
}
```

### Path Security Failures

**Scenario:** Subdirectory outside root bounds

**Response:**
```json
{
  "warnings": [
    "Skipped '/music/../../etc': security validation failed"
  ]
}
```

### Nonexistent Path

**Scenario:** Requested path doesn't exist

**Response:**
```json
{
  "isError": true,
  "message": "Path security validation failed: path does not exist"
}
```

## Use Cases

### 1. Quick Artist/Album Overview

**Goal:** List all albums for an artist without individual tracks

```json
{
  "path": "/music/Artist Name",
  "recursive_depth": 1
}
```

**Result:** Shows album directories only, skipping track files

### 2. Complete Album Track Listing

**Goal:** Get all tracks with file sizes for storage analysis

```json
{
  "path": "/music/Artist Name/Album",
  "recursive_depth": 1,
  "detailed": true
}
```

**Result:** Shows all tracks with exact byte sizes

### 3. Library Structure Discovery

**Goal:** Understand how the library is organized

```json
{
  "path": "/music",
  "recursive_depth": 2
}
```

**Result:** Shows artist → album structure without individual tracks

### 4. Hidden File Detection

**Goal:** Find `.AppleDouble`, `.DS_Store`, or other hidden files

```json
{
  "path": "/music",
  "recursive_depth": -1,
  "include_hidden": true
}
```

**Result:** Discovers all hidden files in library

### 5. Batch Processing Setup

**Goal:** Get list of all audio files for metadata processing

```json
{
  "path": "/music/Artist Name",
  "recursive_depth": -1,
  "detailed": false
}
```

**Result:** Complete list of all files (up to limits) for batch operations

## Performance Considerations

### Execution Time

| Depth | Files | Typical Time | Notes |
|-------|-------|--------------|-------|
| 0 | 10 | ~1ms | Very fast |
| 1 | 100 | ~5ms | Fast |
| 2 | 1000 | ~50ms | Moderate |
| -1 | 1000+ | ~50ms | Capped by limits |

### Memory Usage

- ~100 bytes per entry in result set
- 1000 entries ≈ 100KB memory
- Negligible impact on system

### Optimization Tips

1. **Use appropriate depth:**
   - Don't use `-1` when `2` suffices
   - Start shallow and go deeper if needed

2. **Skip detailed mode when not needed:**
   - Adds minimal overhead, but why compute if not used?

3. **Filter hidden files:**
   - Setting `include_hidden: false` speeds up traversal

4. **Target specific directories:**
   - List `/music/Artist/Album` instead of `/music` when possible

## Security

### Path Validation

Every path is validated against configured security constraints:

```rust
// Each directory validated before traversal
validate_path(&entry_path, config)?
```

**Prevents:**
- ✅ Path traversal attacks (`../../../etc/passwd`)
- ✅ Escaping root directory bounds
- ✅ Following malicious symlinks outside root

### Symlink Loop Detection (Unix)

On Unix systems, the tool tracks visited inodes:

```rust
// Detect circular symlinks
if !visited_inodes.insert(inode) {
    // Loop detected, skip
}
```

**Result:** Cannot be trapped in infinite loops

### Resource Limits

Strict limits prevent resource exhaustion:

- **Max depth:** 10 levels
- **Max entries:** 1000 items

**Result:** Cannot consume excessive memory or CPU

## Implementation Details

**Source:** [`src/domains/tools/definitions/fs/list_dir.rs`](../../../src/domains/tools/definitions/fs/list_dir.rs)

**Key Features:**
- JSON output via `serde_json`
- Recursive traversal with depth tracking
- Alphabetical sorting for consistent results
- Security validation at each level
- Graceful error handling with warnings
- Thread-safe (can be called concurrently)

**Algorithm:**
1. Validate root path
2. Initialize tracking structures (entries, warnings, visited inodes)
3. Call recursive `traverse_directory()` function
4. Sort and serialize results to JSON
5. Return structured response

## Testing

Comprehensive test coverage ensures reliability:

```bash
cargo test --features all list_dir --lib
# Result: 9 tests passed
```

**Test Coverage:**
- ✅ Non-recursive (depth=0)
- ✅ Single level (depth=1)
- ✅ Multi-level (depth=2)
- ✅ Hidden files filtering
- ✅ Detailed mode with sizes
- ✅ Error handling (nonexistent paths)
- ✅ HTTP handlers
- ✅ JSON parsing validation

## Troubleshooting

### Issue: Empty Results

**Symptom:**
```json
{
  "entries": [],
  "dir_count": 0,
  "file_count": 0
}
```

**Possible Causes:**
1. Directory is actually empty
2. All files are hidden and `include_hidden: false`
3. Permission issues (check warnings)

**Solution:** Set `include_hidden: true` and check warnings

### Issue: Truncated Results

**Symptom:**
```json
{
  "warnings": ["Results truncated: exceeded maximum of 1000 entries"]
}
```

**Solution:** Reduce `recursive_depth` or target a smaller directory

### Issue: Missing Subdirectories

**Symptom:** Expected subdirectories not in results

**Possible Causes:**
1. `recursive_depth` too low
2. Permission denied (check warnings)
3. Path outside security root (check warnings)

**Solution:** Increase depth and check warnings array

## Related Tools

- **[fs_rename](fs_rename.md)** - Rename discovered files
- **[read_metadata](../metadata/read_metadata.md)** - Read tags from discovered files
- **[write_metadata](../metadata/write_metadata.md)** - Update tags on discovered files

## Related Documentation

- [Path Security](../../reference/path-security.md) - Security implementation
- [Configuration Guide](../../guides/configuration.md) - Environment setup
- [Recursive Examples](../../examples/fs_list_dir_recursive.md) - More examples
