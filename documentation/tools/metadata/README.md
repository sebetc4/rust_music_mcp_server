# Metadata Tools

This directory contains documentation for audio metadata manipulation tools in the Music MCP Server.

## Available Tools

### Core Operations
- **[read_metadata](read_metadata.md)** - Read audio file tags and technical properties
- **[write_metadata](write_metadata.md)** - Write or update audio file tags

## Quick Comparison

| Tool | Purpose | Modifies File | Include Properties | Output Format |
|------|---------|---------------|-------------------|---------------|
| [read_metadata](read_metadata.md) | Read tags | ❌ No | Optional | JSON |
| [write_metadata](write_metadata.md) | Write/update tags | ✅ Yes | N/A | JSON |

## Supported Audio Formats

All metadata tools support the following formats:

| Format | Extensions | Tag System | Read | Write |
|--------|-----------|------------|------|-------|
| **MP3** | `.mp3` | ID3v1, ID3v2 | ✅ | ✅ ID3v2.4 |
| **FLAC** | `.flac` | Vorbis Comments | ✅ | ✅ |
| **M4A/AAC** | `.m4a`, `.mp4` | iTunes/MP4 | ✅ | ✅ |
| **Opus** | `.opus` | Vorbis Comments | ✅ | ✅ |
| **Vorbis** | `.ogg` | Vorbis Comments | ✅ | ✅ |
| **WAV** | `.wav` | RIFF INFO, ID3 | ✅ | ✅ |
| **AIFF** | `.aiff`, `.aif` | ID3, RIFF | ✅ | ✅ |

**Note**: All tools use the `lofty` library for metadata operations.

## Common Use Cases

### 1. Inspect File Metadata

```json
// Read basic tags
{
  "tool": "read_metadata",
  "path": "/music/artist/album/track.mp3",
  "include_properties": false
}

// Result: Shows title, artist, album, year, track, genre, etc.
```

### 2. Analyze Audio Quality

```json
// Read with technical properties
{
  "tool": "read_metadata",
  "path": "/music/track.flac",
  "include_properties": true
}

// Result: Shows bitrate, sample rate, duration, bit depth, channels
```

### 3. Update Tags from MusicBrainz

```json
// Step 1: Identify file
{"tool": "mb_identify_record", "path": "/music/unknown.mp3"}

// Step 2: Get release info
{"tool": "mb_release_search", "mbid": "release-mbid"}

// Step 3: Write metadata
{
  "tool": "write_metadata",
  "path": "/music/unknown.mp3",
  "title": "Song Title",
  "artist": "Artist Name",
  "album": "Album Name",
  "year": 2024,
  "track": 5
}

// Step 4: Verify
{"tool": "read_metadata", "path": "/music/unknown.mp3"}
```

### 4. Batch Update Album

```json
// Update common album fields for all tracks
{
  "tool": "write_metadata",
  "path": "/music/artist/album/01 - track.mp3",
  "album": "Album Name",
  "album_artist": "Artist Name",
  "year": 2024,
  "genre": "Rock"
}

// Repeat for each track, updating track number individually
```

### 5. Find Untagged Files

```json
// Step 1: List files
{"tool": "fs_list_dir", "path": "/music/folder", "recursive_depth": 1}

// Step 2: Read metadata for each file
{"tool": "read_metadata", "path": "/music/folder/file.mp3"}

// Step 3: If metadata is null or incomplete, use MusicBrainz
{"tool": "mb_identify_record", "path": "/music/folder/file.mp3"}
```

## Available Metadata Fields

Both tools support these standard metadata fields:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `title` | string | Track title | "Bohemian Rhapsody" |
| `artist` | string | Track artist/performer | "Queen" |
| `album` | string | Album name | "A Night at the Opera" |
| `album_artist` | string | Album artist | "Queen" |
| `year` | integer | Release year | 1975 |
| `track` | integer | Track number | 11 |
| `track_total` | integer | Total tracks in album | 12 |
| `genre` | string | Music genre | "Progressive Rock" |
| `comment` | string | Comment/description | "Remastered 2011" |

### Technical Properties (read_metadata only)

When `include_properties: true`:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `duration_seconds` | integer | Total duration in seconds | 355 |
| `duration_formatted` | string | Human-readable duration | "5:55" |
| `bitrate_kbps` | integer | Audio bitrate | 320 |
| `sample_rate_hz` | integer | Sample rate in Hz | 44100 |
| `channels` | integer | Number of channels | 2 |
| `channel_description` | string | Channel layout | "Stereo" |
| `bit_depth` | integer | Bits per sample | 16 |

## Tool Integration Workflow

```
┌─────────────────────────────────────────────────────────────┐
│                    Metadata Workflow                         │
└─────────────────────────────────────────────────────────────┘

fs_list_dir → Discover audio files
     ↓
read_metadata → Check existing tags
     ↓
┌────┴─────────────────────────────┐
│ Tags OK?                          │
├─ Yes → fs_rename (if needed)      │
└─ No → Continue to correction      │
     ↓
mb_identify_record → Fingerprint file
     ↓
mb_release_search → Get correct metadata
     ↓
write_metadata → Update tags
     ↓
read_metadata → Verify changes
     ↓
fs_rename → Organize by new tags
```

## Security & Safety

All metadata tools implement:

- ✅ **Path validation** against configured root directory
- ✅ **Security constraints** prevent path traversal attacks
- ✅ **Error handling** with graceful degradation
- ✅ **Detailed logging** for audit trails

### Read-Only vs. Write Operations

| Tool | File Modification | Reversible | Risk Level |
|------|------------------|------------|------------|
| read_metadata | ❌ None | N/A | 🟢 Low (read-only) |
| write_metadata | ✅ Metadata only | ⚠️ Partial* | 🟡 Medium (data modification) |

*Write operations modify file metadata permanently. Audio data is preserved, but original tags are overwritten (especially with `clear_existing: true`).

See [Path Security](../../reference/path-security.md) for implementation details.

## Best Practices

### 1. Always Read Before Writing

```json
// Bad: Blind write
{"tool": "write_metadata", "path": "/file.mp3", "title": "New"}

// Good: Read first, then update
{"tool": "read_metadata", "path": "/file.mp3"}
// ... analyze ...
{"tool": "write_metadata", "path": "/file.mp3", "title": "New"}
```

### 2. Request Properties Only When Needed

```json
// Bad: Always include properties
{"path": "/file.mp3", "include_properties": true}

// Good: Include only when analyzing quality
{"path": "/file.mp3", "include_properties": false}  // Default
```

Properties add minimal overhead, but omitting them keeps responses concise.

### 3. Partial Updates Over Full Rewrites

```json
// Bad: Rewrite everything
{
  "path": "/file.mp3",
  "clear_existing": true,
  "title": "Title",
  "artist": "Artist",
  "album": "Album"
}

// Good: Update only what changed
{
  "path": "/file.mp3",
  "title": "Corrected Title"
}
```

Partial updates preserve album art, ReplayGain, and other specialized tags.

### 4. Verify Changes

```typescript
// Write
await write_metadata({path, title: "New Title"});

// Verify
const result = await read_metadata({path});
console.assert(result.metadata.title === "New Title");
```

### 5. Handle Missing Metadata Gracefully

```typescript
const result = await read_metadata({path: filePath});

if (result.structuredContent.metadata === null) {
  // No tags found - use mb_identify_record
  await fingerprint_and_tag(filePath);
} else if (!result.structuredContent.metadata.title) {
  // Partial tags - may need correction
  await verify_and_correct(filePath);
}
```

## Performance Considerations

### Read Performance

| Format | Typical Read Time | Notes |
|--------|------------------|-------|
| MP3 | < 50ms | ID3 tags at file start |
| FLAC | < 50ms | Comments in header |
| M4A | < 100ms | Atom-based (random access) |
| WAV | < 30ms | Small chunks |

### Write Performance

| Operation | Typical Write Time | Notes |
|-----------|-------------------|-------|
| Single field | < 50ms | In-place update |
| Multiple fields | < 100ms | Single write |
| Clear + rewrite | < 150ms | Remove + create |

**Recommendation**: Both operations are fast. For batch processing:
- Process files sequentially to avoid I/O saturation
- Use error handling for each file
- Consider rate limiting for 1000+ files

## Limitations

### What These Tools CAN Do

- ✅ Read/write standard metadata fields
- ✅ Get technical audio properties
- ✅ Support all major audio formats
- ✅ Preserve audio data during writes

### What These Tools CANNOT Do

- ❌ **Album artwork** - Cannot read or write cover images
- ❌ **Lyrics** - Cannot extract or write lyrics tags
- ❌ **ReplayGain** - Cannot read or write ReplayGain tags
- ❌ **Custom tags** - Only standard tags supported
- ❌ **Embedded cue sheets** - Not extracted or written

For these features, consider specialized tools or libraries.

## Configuration

Set root directory constraints via environment variables:

```bash
# Restrict metadata operations to this directory
export MCP_ROOT_PATH="/music"

# Allow symlink following (use with caution)
export MCP_ALLOW_SYMLINKS=false  # Default
```

See [Configuration Guide](../../guides/configuration.md) for details.

## Error Handling

### Common Errors

| Error Message | Cause | Solution |
|--------------|-------|----------|
| "Path is not a file" | Path is directory or doesn't exist | Verify path with fs_list_dir |
| "Failed to read audio file" | Unsupported format or corrupted | Check file format |
| "Permission denied" | No write access | Check file permissions |
| "Path security validation failed" | Path outside root | Use path within MCP_ROOT_PATH |

### Error Response Format

All errors return:

```json
{
  "content": [
    {
      "type": "text",
      "text": "Descriptive error message"
    }
  ],
  "isError": true
}
```

## Related Documentation

### Filesystem Tools
- [fs_list_dir](../fs/fs_list_dir.md) - Discover audio files
- [fs_rename](../fs/fs_rename.md) - Rename files after tagging
- [fs_delete](../fs/fs_delete.md) - Remove duplicate files

### MusicBrainz Tools
- [mb_identify_record](../mb/mb_identify_record.md) - Audio fingerprinting
- [mb_release_search](../mb/mb_release_search.md) - Get album metadata
- [mb_recording_search](../mb/mb_recording_search.md) - Search recordings
- [mb_artist_search](../mb/mb_artist_search.md) - Search artists

### Reference
- [Tool Output Formats](../../reference/tool-output-formats.md) - MCP output guide
- [Path Security](../../reference/path-security.md) - Security implementation
- [Error Handling](../../reference/error-handling.md) - Error patterns

### Guides
- [Adding New Tools](../../guides/adding-tools.md) - Extend metadata capabilities
- [Configuration](../../guides/configuration.md) - Setup and config
- [Testing](../../guides/testing.md) - Testing strategies

## Tool-Specific Documentation

- [read_metadata.md](read_metadata.md) - Detailed `read_metadata` documentation
- [write_metadata.md](write_metadata.md) - Detailed `write_metadata` documentation

## Implementation Details

**Source Code**:
- Read: [src/domains/tools/definitions/metadata/read.rs](../../../src/domains/tools/definitions/metadata/read.rs)
- Write: [src/domains/tools/definitions/metadata/write.rs](../../../src/domains/tools/definitions/metadata/write.rs)

**Key Dependencies**:
- `lofty` 0.22.4 - Audio metadata reading/writing library

**Transport Support**:
- ✅ STDIO (default)
- ✅ TCP
- ✅ HTTP

**Test Coverage**:
- Unit tests for both read and write operations
- Error handling tests
- HTTP handler tests (when `http` feature enabled)
