# write_metadata

Write or update audio file metadata (ID3 tags, Vorbis comments, etc.) for various audio formats. Returns structured JSON confirming updates.

## Overview

The `write_metadata` tool writes metadata tags to audio files using the `lofty` library, making it ideal for:

- ✏️ Updating audio file tags (title, artist, album, etc.)
- 🏷️ Correcting incorrect metadata
- 📝 Adding missing tags to untagged files
- 🔄 Standardizing music library organization

## Supported Formats

| Format | Extensions | Tag Types Written |
|--------|------------|-------------------|
| **MP3** | `.mp3` | ID3v2.4 (primary) |
| **FLAC** | `.flac` | Vorbis Comments |
| **M4A/AAC** | `.m4a`, `.mp4` | iTunes/MP4 tags |
| **Opus** | `.opus` | Vorbis Comments |
| **Vorbis** | `.ogg` | Vorbis Comments |
| **WAV** | `.wav` | ID3v2, RIFF INFO |
| **AIFF** | `.aiff`, `.aif` | ID3v2 |

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path` | string | ✅ Yes | - | Path to the audio file to modify |
| `title` | string | ❌ No | - | Track title |
| `artist` | string | ❌ No | - | Artist/performer name |
| `album` | string | ❌ No | - | Album name |
| `album_artist` | string | ❌ No | - | Album artist (may differ from track artist) |
| `year` | integer | ❌ No | - | Release year |
| `track` | integer | ❌ No | - | Track number |
| `track_total` | integer | ❌ No | - | Total tracks in album |
| `genre` | string | ❌ No | - | Music genre |
| `comment` | string | ❌ No | - | Comment/description |
| `clear_existing` | boolean | ❌ No | `false` | Clear all existing tags before writing |

### Update Behavior

- **Partial updates**: Only provided fields are updated
- **Existing tags preserved**: Fields not specified remain unchanged (unless `clear_existing: true`)
- **Creates tags if missing**: Tool creates a new tag if file has none
- **Format-specific tags**: Uses appropriate tag format for each file type

## Output Format

Returns structured JSON with update confirmation:

```json
{
  "file": "/music/artist/album/track.mp3",
  "clear_existing": false,
  "fields_updated": 4,
  "updated_fields": {
    "title": "New Title",
    "artist": "New Artist",
    "album": "New Album",
    "year": "2024"
  }
}
```

### Output Fields

- **`file`**: Path to the file that was updated (echoes request)
- **`clear_existing`**: Whether existing tags were cleared (echoes request)
- **`fields_updated`**: Number of fields updated (integer)
- **`updated_fields`**: Map of field names to new values
  - Keys: `"title"`, `"artist"`, `"album"`, `"album_artist"`, `"year"`, `"track"`, `"track_total"`, `"genre"`, `"comment"`
  - Values: String representation of new value

### MCP Output Format

This tool follows MCP best practices by returning data in two forms:

1. **Text Summary** (human-readable):
   - Normal: `"Updated 4 field(s) in '/music/track.mp3': title, artist, album, year"`
   - Clear: `"Cleared and updated 3 field(s) in '/music/track.mp3': title, artist, year"`
   - No updates: `"No fields updated for '/music/track.mp3'"`
2. **Structured Content** (machine-readable): The JSON structure shown above

AI agents can directly parse the `structuredContent` field for programmatic verification.

For more information on MCP output formats, see [Tool Output Formats Guide](../../reference/tool-output-formats.md).

## Examples

### Update Multiple Fields

**Request:**
```json
{
  "path": "/music/track.mp3",
  "title": "Bohemian Rhapsody",
  "artist": "Queen",
  "album": "A Night at the Opera",
  "year": 1975,
  "track": 11
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Updated 5 field(s) in '/music/track.mp3': title, artist, album, year, track"
    }
  ],
  "structuredContent": {
    "file": "/music/track.mp3",
    "clear_existing": false,
    "fields_updated": 5,
    "updated_fields": {
      "title": "Bohemian Rhapsody",
      "artist": "Queen",
      "album": "A Night at the Opera",
      "year": "1975",
      "track": "11"
    }
  },
  "isError": false
}
```

### Update Single Field

**Request:**
```json
{
  "path": "/music/artist/album/03 - track.flac",
  "track": 3
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Updated 1 field(s) in '/music/artist/album/03 - track.flac': track"
    }
  ],
  "structuredContent": {
    "file": "/music/artist/album/03 - track.flac",
    "clear_existing": false,
    "fields_updated": 1,
    "updated_fields": {
      "track": "3"
    }
  },
  "isError": false
}
```

### Clear and Rewrite Tags

**Request:**
```json
{
  "path": "/music/track.mp3",
  "title": "Clean Title",
  "artist": "Clean Artist",
  "clear_existing": true
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Cleared and updated 2 field(s) in '/music/track.mp3': title, artist"
    }
  ],
  "structuredContent": {
    "file": "/music/track.mp3",
    "clear_existing": true,
    "fields_updated": 2,
    "updated_fields": {
      "title": "Clean Title",
      "artist": "Clean Artist"
    }
  },
  "isError": false
}
```

## Error Handling

The tool provides clear error messages for common issues:

### File Not Found

```json
{
  "content": [
    {
      "type": "text",
      "text": "Path is not a file: /music/nonexistent.mp3"
    }
  ],
  "isError": true
}
```

### Path is a Directory

```json
{
  "content": [
    {
      "type": "text",
      "text": "Path is not a file: /music/artist"
    }
  ],
  "isError": true
}
```

### Unsupported Format

```json
{
  "content": [
    {
      "type": "text",
      "text": "Failed to read audio file: Unsupported file format"
    }
  ],
  "isError": true
}
```

### Write Permission Error

```json
{
  "content": [
    {
      "type": "text",
      "text": "Failed to save metadata: Permission denied"
    }
  ],
  "isError": true
}
```

### Path Security Violation

```json
{
  "content": [
    {
      "type": "text",
      "text": "Path security validation failed: Path is outside allowed root directory"
    }
  ],
  "isError": true
}
```

## Use Cases

### Correct Metadata from MusicBrainz

```json
// Step 1: Identify file
{"tool": "mb_identify_record", "path": "/music/unknown.mp3"}

// Step 2: Get release details
{"tool": "mb_release_search", "mbid": "release-id-from-step1"}

// Step 3: Write correct metadata
{
  "tool": "write_metadata",
  "path": "/music/unknown.mp3",
  "title": "Correct Title",
  "artist": "Correct Artist",
  "album": "Correct Album",
  "year": 2020,
  "track": 5
}

// Step 4: Verify changes
{"tool": "read_metadata", "path": "/music/unknown.mp3"}
```

### Batch Update Album

```typescript
const albumPath = "/music/Artist/Album";
const albumInfo = {
  album: "Album Name",
  album_artist: "Artist Name",
  year: 2024,
  genre: "Rock"
};

// List all tracks
const tracks = await mcpClient.callTool("fs_list_dir", {
  path: albumPath,
  recursive_depth: 0
});

// Update each track
for (const track of tracks.structuredContent.entries) {
  if (track.type === "file" && track.name.endsWith(".mp3")) {
    await mcpClient.callTool("write_metadata", {
      path: `${albumPath}/${track.name}`,
      ...albumInfo,
      track: extractTrackNumber(track.name)
    });
  }
}
```

### Remove Unwanted Tags

```json
{
  "path": "/music/track.mp3",
  "title": "Clean Title",
  "artist": "Artist",
  "album": "Album",
  "clear_existing": true
}

// All other tags (genre, comment, etc.) are removed
```

### Add Missing Track Numbers

```json
{
  "path": "/music/album/01 - Song.mp3",
  "track": 1,
  "track_total": 12
}
```

## Integration with Other Tools

### Complete Metadata Workflow

```
fs_list_dir → Discover untagged files
     ↓
read_metadata → Check existing tags
     ↓
mb_identify_record → Fingerprint to find correct metadata
     ↓
mb_release_search → Get complete album info
     ↓
write_metadata → Update tags with correct information
     ↓
read_metadata → Verify changes
     ↓
fs_rename → Rename file based on new tags
```

### Quality Control

```
read_metadata → Identify poorly tagged files
     ↓
(AI agent analyzes quality)
     ↓
write_metadata → Correct/standardize tags
     ↓
read_metadata → Confirm updates
```

## Best Practices

### 1. Read Before Writing

```json
// Bad: Blindly overwrite
{"tool": "write_metadata", "path": "/file.mp3", "title": "New Title"}

// Good: Read first to preserve other tags
{"tool": "read_metadata", "path": "/file.mp3"}
// ... check what exists ...
{"tool": "write_metadata", "path": "/file.mp3", "title": "New Title"}
```

### 2. Update Only What's Needed

```json
// Bad: Update everything even if correct
{
  "path": "/file.mp3",
  "title": "Title",
  "artist": "Artist",
  "album": "Album",
  "year": 2024,
  "track": 1,
  "genre": "Rock"
}

// Good: Update only incorrect fields
{
  "path": "/file.mp3",
  "title": "Corrected Title"
}
```

### 3. Use clear_existing Sparingly

```json
// Bad: Always clear tags
{"path": "/file.mp3", "clear_existing": true, "title": "Title"}

// Good: Clear only when necessary (e.g., removing junk tags)
{"path": "/file.mp3", "clear_existing": false, "title": "Title"}
```

**Why**: Clearing removes ALL tags including album art, ReplayGain, and other specialized tags.

### 4. Verify Changes

```typescript
// Write metadata
const writeResult = await mcpClient.callTool("write_metadata", {
  path: filePath,
  title: "New Title",
  artist: "New Artist"
});

// Verify changes
const readResult = await mcpClient.callTool("read_metadata", {
  path: filePath
});

const metadata = readResult.structuredContent.metadata;
console.assert(metadata.title === "New Title");
console.assert(metadata.artist === "New Artist");
```

### 5. Handle Errors Gracefully

```typescript
try {
  const result = await mcpClient.callTool("write_metadata", params);

  if (result.isError) {
    if (result.content[0].text.includes("Permission denied")) {
      console.error("Cannot write: check file permissions");
    } else if (result.content[0].text.includes("Unsupported")) {
      console.error("File format not supported");
    }
  } else {
    console.log(`Updated ${result.structuredContent.fields_updated} fields`);
  }
} catch (error) {
  console.error("Write failed:", error);
}
```

## Security & Safety

### Path Validation

All write operations are subject to strict path security:

- ✅ **Root directory enforcement** - Cannot write outside configured root
- ✅ **Path traversal prevention** - Blocks `..` and symlink attacks
- ✅ **File validation** - Ensures target is a file, not directory

See [Path Security Reference](../../reference/path-security.md) for details.

### Data Safety

- **Atomic writes**: `lofty` attempts atomic tag updates
- **Preserves audio data**: Only metadata is modified, audio stream untouched
- **Format-specific handling**: Uses correct tag format for each file type
- **Backup recommendation**: Consider backups before bulk updates

### Privacy Considerations

- **Comment field**: Be cautious about storing personal info
- **Custom tags**: Tool doesn't write custom/proprietary tags
- **Logging**: Metadata changes logged at INFO level

## Performance

| Operation | Speed | Notes |
|-----------|-------|-------|
| Single field update | Fast (< 50ms) | In-place tag update |
| Multiple fields | Fast (< 100ms) | Single write operation |
| Clear + rewrite | Moderate (< 150ms) | Tag removal + creation |

**Recommendation**: Write operations are generally fast. For batch updates on thousands of files:
- Process files sequentially to avoid I/O contention
- Include error handling for each file
- Consider progress tracking for user feedback

## Comparison with Other Tools

| Tool | Purpose | Modifies File | Source |
|------|---------|---------------|--------|
| [write_metadata](write_metadata.md) | **Update** tags | ✅ Yes | User input |
| [read_metadata](read_metadata.md) | **Read** tags | ❌ No | File |
| [mb_identify_record](../mb/mb_identify_record.md) | **Identify** via fingerprint | ❌ No | MusicBrainz |
| [mb_release_search](../mb/mb_release_search.md) | **Get** metadata | ❌ No | MusicBrainz |

**Workflow**: `mb_identify_record` or `mb_release_search` → `write_metadata`

## Limitations

- ❌ **No artwork writing** - Cannot write/update cover images (lofty limitation)
- ❌ **No lyrics** - Cannot write synchronized lyrics
- ❌ **No ReplayGain** - Cannot write ReplayGain tags
- ❌ **No custom tags** - Only standard tags supported
- ✅ **Primary tag only** - Writes to primary tag format for each file type

## Tag Format Details

### MP3 (ID3v2.4)
- **Written as**: ID3v2.4 tags
- **Encoding**: UTF-8
- **Frames**: Standard ID3v2.4 frames (TIT2, TPE1, TALB, etc.)

### FLAC (Vorbis Comments)
- **Written as**: Vorbis Comments
- **Format**: `FIELD=value`
- **Standard fields**: TITLE, ARTIST, ALBUM, DATE, TRACKNUMBER, etc.

### M4A (iTunes/MP4)
- **Written as**: iTunes-compatible atoms
- **Atoms**: `©nam`, `©ART`, `©alb`, `©day`, `trkn`, etc.

### WAV
- **Primary**: ID3v2 tags (preferred)
- **Fallback**: RIFF INFO chunks

## Configuration

The write tool respects global path security configuration:

```bash
# Set root directory (writes restricted to this path)
export MCP_ROOT_PATH="/music"

# Allow symlink following
export MCP_ALLOW_SYMLINKS=false  # Default: false
```

See [Configuration Guide](../../guides/configuration.md) for details.

## Related Documentation

- [read_metadata](read_metadata.md) - Read audio file tags
- [mb_identify_record](../mb/mb_identify_record.md) - Identify files via fingerprint
- [mb_release_search](../mb/mb_release_search.md) - Get metadata from MusicBrainz
- [fs_rename](../fs/fs_rename.md) - Rename files after updating tags
- [Path Security](../../reference/path-security.md) - Security implementation
- [Tool Output Formats](../../reference/tool-output-formats.md) - MCP output guide

## Implementation Details

**Source Code**: [src/domains/tools/definitions/metadata/write.rs](../../../src/domains/tools/definitions/metadata/write.rs)

**Key Dependencies**:
- `lofty` 0.22.4 - Audio metadata library
- Supports: ID3v1/v2, Vorbis Comments, APE, iTunes/MP4 tags

**Key Features**:
- Path security validation before write
- Automatic format detection
- Partial update support (only specified fields)
- Clear existing option for clean rewrites
- Tag creation if none exist
- Comprehensive error handling
- Full test coverage

**Transport Support**:
- ✅ STDIO (default)
- ✅ TCP
- ✅ HTTP

## Troubleshooting

### "Failed to save metadata"

**Causes**:
- File is read-only or locked
- Insufficient disk space
- File corruption

**Solutions**:
- Check file permissions: `ls -l file.mp3`
- Verify disk space: `df -h`
- Try with a different file to isolate issue

### Tags Not Persisting

**Causes**:
- File format doesn't support tag type
- Media player caching old metadata

**Solutions**:
- Verify format support (see Supported Formats table)
- Refresh media player library/cache
- Use `read_metadata` to confirm tags were written

### clear_existing Removes Too Much

**Cause**: `clear_existing: true` removes ALL tags including artwork

**Solution**: Don't use `clear_existing` unless you want to remove everything. For selective removal, read → modify → write specific fields.
