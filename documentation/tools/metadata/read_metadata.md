# read_metadata

Read audio file metadata (ID3 tags, Vorbis comments, etc.) from various audio formats. Returns structured JSON optimized for AI agents.

## Overview

The `read_metadata` tool reads metadata tags from audio files using the `lofty` library, making it ideal for:

- 🎵 Inspecting audio file tags (title, artist, album, etc.)
- 📊 Analyzing music library organization
- 🔍 Identifying untagged or poorly tagged files
- ⚙️ Extracting technical audio properties (bitrate, duration, sample rate)

## Supported Formats

| Format | Extensions | Tag Types |
|--------|------------|-----------|
| **MP3** | `.mp3` | ID3v1, ID3v2 |
| **FLAC** | `.flac` | Vorbis Comments |
| **M4A/AAC** | `.m4a`, `.mp4` | iTunes/MP4 tags |
| **Opus** | `.opus` | Vorbis Comments |
| **Vorbis** | `.ogg` | Vorbis Comments |
| **WAV** | `.wav` | RIFF INFO, ID3 |
| **AIFF** | `.aiff`, `.aif` | ID3, RIFF |

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path` | string | ✅ Yes | - | Path to the audio file to read |
| `include_properties` | boolean | ❌ No | `false` | Include technical audio properties |

## Output Format

Returns structured JSON with metadata and optional properties:

```json
{
  "file": "/music/artist/album/track.mp3",
  "format": "Mpeg",
  "metadata": {
    "title": "Song Title",
    "artist": "Artist Name",
    "album": "Album Name",
    "album_artist": "Album Artist",
    "year": 2024,
    "track": 3,
    "genre": "Rock",
    "comment": "Purchased from...",
    "total_tags": 15
  },
  "properties": {  // Only if include_properties: true
    "duration_seconds": 245,
    "duration_formatted": "4:05",
    "bitrate_kbps": 320,
    "sample_rate_hz": 44100,
    "channels": 2,
    "channel_description": "Stereo",
    "bit_depth": 16
  }
}
```

### Metadata Fields

All metadata fields are **optional** and present only if found in the file:

- **`title`**: Track title
- **`artist`**: Primary artist/performer
- **`album`**: Album name
- **`album_artist`**: Album artist (may differ from track artist)
- **`year`**: Release year (unsigned integer)
- **`track`**: Track number (unsigned integer)
- **`genre`**: Music genre
- **`comment`**: Comment/description field
- **`total_tags`**: Total number of tags found in file (always present)

### Properties Fields

Technical audio properties (only when `include_properties: true`):

- **`duration_seconds`**: Total duration in seconds
- **`duration_formatted`**: Human-readable duration (e.g., "3:45")
- **`bitrate_kbps`**: Audio bitrate in kilobits per second
- **`sample_rate_hz`**: Sample rate in Hertz (e.g., 44100, 48000)
- **`channels`**: Number of audio channels (1, 2, etc.)
- **`channel_description`**: Human-readable channel info
  - `"Mono"`: 1 channel
  - `"Stereo"`: 2 channels
  - `"Multi-channel"`: 3+ channels
- **`bit_depth`**: Bits per sample (e.g., 16, 24)

### MCP Output Format

This tool follows MCP best practices by returning data in two forms:

1. **Text Summary** (human-readable):
   - With properties: `"'Song Title' by Artist Name (4:05, 15 tags)"`
   - Without properties: `"'Song Title' by Artist Name (15 tags)"`
   - No metadata: `"No metadata found in '/path/to/file.mp3'"`
2. **Structured Content** (machine-readable): The JSON structure shown above

AI agents can directly parse the `structuredContent` field for programmatic access.

For more information on MCP output formats, see [Tool Output Formats Guide](../../reference/tool-output-formats.md).

## Examples

### Read Basic Metadata

**Request:**
```json
{
  "path": "/music/Pink Floyd/The Wall/01 - In The Flesh.mp3",
  "include_properties": false
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "'In The Flesh?' by Pink Floyd (15 tags)"
    }
  ],
  "structuredContent": {
    "file": "/music/Pink Floyd/The Wall/01 - In The Flesh.mp3",
    "format": "Mpeg",
    "metadata": {
      "title": "In The Flesh?",
      "artist": "Pink Floyd",
      "album": "The Wall",
      "album_artist": "Pink Floyd",
      "year": 1979,
      "track": 1,
      "genre": "Progressive Rock",
      "total_tags": 15
    },
    "properties": null
  },
  "isError": false
}
```

### Read Metadata with Properties

**Request:**
```json
{
  "path": "/music/track.flac",
  "include_properties": true
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "'Example Song' by Example Artist (3:42, 8 tags)"
    }
  ],
  "structuredContent": {
    "file": "/music/track.flac",
    "format": "Flac",
    "metadata": {
      "title": "Example Song",
      "artist": "Example Artist",
      "album": "Example Album",
      "year": 2023,
      "track": 5,
      "total_tags": 8
    },
    "properties": {
      "duration_seconds": 222,
      "duration_formatted": "3:42",
      "bitrate_kbps": 1024,
      "sample_rate_hz": 44100,
      "channels": 2,
      "channel_description": "Stereo",
      "bit_depth": 16
    }
  },
  "isError": false
}
```

### File with No Metadata

**Request:**
```json
{
  "path": "/music/untagged.mp3",
  "include_properties": false
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "No metadata found in '/music/untagged.mp3'"
    }
  ],
  "structuredContent": {
    "file": "/music/untagged.mp3",
    "format": "Mpeg",
    "metadata": null,
    "properties": null
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

### Verify Music Library Tags

```json
// Step 1: List all files
{"tool": "fs_list_dir", "path": "/music/artist/album", "recursive_depth": 1}

// Step 2: Read metadata for each file
{"tool": "read_metadata", "path": "/music/artist/album/01 - track.mp3"}

// Step 3: Identify files with missing tags
// (AI agent checks if title/artist/album are present)
```

### Analyze Audio Quality

```json
{
  "path": "/music/high-res/track.flac",
  "include_properties": true
}

// Check bitrate_kbps and bit_depth to verify high-quality audio
```

### Compare File Metadata

```json
// Read original file
{"tool": "read_metadata", "path": "/music/original.mp3"}

// Read potential duplicate
{"tool": "read_metadata", "path": "/music/duplicate.mp3"}

// Compare title, artist, duration to identify duplicates
```

### Identify Untagged Files

```typescript
const result = await mcpClient.callTool("read_metadata", {
  path: filePath,
  include_properties: false
});

const metadata = result.structuredContent.metadata;

if (!metadata || !metadata.title || !metadata.artist) {
  console.log(`Untagged file: ${filePath}`);
  // Use mb_identify_record to fingerprint and find metadata
}
```

## Integration with Other Tools

### Workflow: Read → Identify → Write

```
1. read_metadata → Check existing tags
        ↓
2. mb_identify_record → Fingerprint if tags missing/incorrect
        ↓
3. mb_release_search → Get correct metadata from MusicBrainz
        ↓
4. write_metadata → Update tags with correct information
        ↓
5. read_metadata → Verify changes
```

### Discovery and Analysis

```
fs_list_dir → Discover audio files
     ↓
read_metadata → Analyze tag quality
     ↓
(filter files with missing/incorrect tags)
     ↓
mb_identify_record → Correct metadata
```

## Best Practices

### 1. Use Properties Sparingly

```json
// Bad: Always include properties
{"path": "/file.mp3", "include_properties": true}

// Good: Only request when needed
{"path": "/file.mp3", "include_properties": false}  // Just tags
```

**Why**: Properties add overhead and are only needed for quality analysis or duration-based filtering.

### 2. Check for Null Metadata

```typescript
const data = result.structuredContent;

if (data.metadata === null) {
  // File has no tags
  console.log("No metadata found");
} else if (!data.metadata.title) {
  // File has some tags but missing title
  console.log("Title tag missing");
}
```

### 3. Batch Reading with Error Handling

```typescript
const files = ["/music/track1.mp3", "/music/track2.mp3"];
const results = [];

for (const file of files) {
  try {
    const result = await mcpClient.callTool("read_metadata", {path: file});
    if (!result.isError) {
      results.push(result.structuredContent);
    }
  } catch (error) {
    console.error(`Failed to read ${file}:`, error);
  }
}
```

### 4. Use total_tags as Quality Indicator

```typescript
const metadata = result.structuredContent.metadata;

if (metadata && metadata.total_tags < 5) {
  console.log("File is poorly tagged (< 5 tags)");
  // Consider using mb_identify_record for better metadata
}
```

## Security & Safety

### Path Validation

All read operations are subject to strict path security:

- ✅ **Root directory enforcement** - Cannot read outside configured root
- ✅ **Path traversal prevention** - Blocks `..` and symlink attacks
- ✅ **Read-only operation** - Never modifies files

See [Path Security Reference](../../reference/path-security.md) for details.

### Privacy Considerations

- **Comment tags** may contain personal information
- **Embedded artwork** is not read (use specialized tools if needed)
- **Metadata is logged** at INFO level (may appear in logs)

## Performance

| Format | Read Speed | Notes |
|--------|------------|-------|
| MP3 | Fast | ID3 tags at file beginning |
| FLAC | Fast | Vorbis comments at beginning |
| M4A | Moderate | Metadata in atoms (random access) |
| WAV | Fast | Small RIFF chunks |

**Recommendation**: Reading metadata is generally fast (< 100ms per file). For batch operations on thousands of files, consider rate limiting to avoid I/O saturation.

## Comparison with Other Tools

| Tool | Purpose | Modifies File | Output |
|------|---------|---------------|--------|
| [read_metadata](read_metadata.md) | **Read** tags | ❌ No | Metadata + properties |
| [write_metadata](write_metadata.md) | **Write/update** tags | ✅ Yes | Updated fields summary |
| [mb_identify_record](../mb/mb_identify_record.md) | **Identify** via fingerprint | ❌ No | MusicBrainz match |

## Limitations

- ❌ **No artwork extraction** - Images/cover art are not returned
- ❌ **No lyrics** - Lyrics tags are not extracted (lofty limitation)
- ❌ **No ReplayGain** - ReplayGain tags not exposed
- ✅ **Primary tag only** - Reads the primary tag type for each format

## Configuration

The read tool respects global path security configuration:

```bash
# Set root directory (reads restricted to this path)
export MCP_ROOT_PATH="/music"

# Allow symlink following
export MCP_ALLOW_SYMLINKS=false  # Default: false
```

See [Configuration Guide](../../guides/configuration.md) for details.

## Related Documentation

- [write_metadata](write_metadata.md) - Update audio file tags
- [mb_identify_record](../mb/mb_identify_record.md) - Identify files via fingerprint
- [fs_list_dir](../fs/fs_list_dir.md) - Discover audio files
- [Path Security](../../reference/path-security.md) - Security implementation
- [Tool Output Formats](../../reference/tool-output-formats.md) - MCP output guide

## Implementation Details

**Source Code**: [src/domains/tools/definitions/metadata/read.rs](../../../src/domains/tools/definitions/metadata/read.rs)

**Key Dependencies**:
- `lofty` 0.22.4 - Audio metadata library
- Supports: ID3v1, ID3v2, Vorbis Comments, APE, iTunes/MP4 tags

**Key Features**:
- Path security validation before read
- Automatic format detection
- Primary tag extraction with fallback
- Optional properties with formatted output
- Comprehensive error handling
- Full test coverage

**Transport Support**:
- ✅ STDIO (default)
- ✅ TCP
- ✅ HTTP
