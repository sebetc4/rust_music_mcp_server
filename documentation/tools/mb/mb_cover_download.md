# mb_cover_download

Download cover art images for music releases from the MusicBrainz Cover Art Archive.

---

## Overview

The `mb_cover_download` tool allows you to:
- Download cover art images for any release in the MusicBrainz database
- Choose from multiple thumbnail sizes (250, 500, 1200 pixels, or original resolution)
- Automatic intelligent fallback if requested size is unavailable
- Prioritizes Front cover art but falls back to other available images
- Returns structured JSON data with file information and metadata

**Output Format**: This tool follows MCP standards, returning a short text summary plus structured JSON data for programmatic access.

---

## Parameters

```typescript
{
  mbid: string,              // MusicBrainz Release ID (UUID) (required)
  path: string,              // Target directory path (required)
  filename?: string,         // Output filename without extension (default: "cover")
  thumbnail_size?: string,   // Size: "250", "500", "1200", or "original" (default: "500")
  overwrite?: boolean        // Overwrite existing file (default: false)
}
```

### Parameter Details

- **mbid** (required)
  - MusicBrainz Release ID in UUID format
  - Example: `"65c70b9f-fdef-4bc0-a5b6-ac4e34252d3c"`
  - Must be exactly 36 characters with dashes at positions 8, 13, 18, 23

- **path** (required)
  - Directory where the cover image will be saved
  - Must be within allowed root directory (security validation)
  - Must be an existing directory (not a file)
  - Example: `"/home/user/music/albums"`

- **filename** (optional)
  - Output filename without extension (extension auto-detected)
  - Default: `"cover"`
  - Example: `"album_art"` will create `album_art.jpg` or `album_art.png`

- **thumbnail_size** (optional)
  - `"250"`: 250px thumbnail
  - `"500"`: 500px thumbnail (default)
  - `"1200"`: 1200px thumbnail
  - `"original"`: Full resolution image
  - Intelligent fallback if requested size unavailable

- **overwrite** (optional)
  - `true`: Replace existing file if present
  - `false`: Return error if file exists (default)

---

## Output Format

The tool returns:
1. **Text Summary**: A concise description of the download (e.g., "Downloaded Front cover (500) to cover.jpg (45231 bytes)")
2. **Structured JSON Data**: Complete data in a standardized format for programmatic access

### Structured Output

```typescript
{
  success: boolean,           // Always true on success
  file_path: string,          // Absolute path to saved file
  file_size_bytes: number,    // Size of downloaded image in bytes
  image_type: string,         // Type of image: "Front", "Back", "Booklet", etc.
  thumbnail_size: string,     // Actual size downloaded: "250", "500", "1200", "original"
  source_url: string          // URL from which image was downloaded
}
```

---

## Examples

### Example 1: Simple Cover Download

Download a 500px front cover for a release.

**Request**:
```json
{
  "name": "mb_cover_download",
  "arguments": {
    "mbid": "65c70b9f-fdef-4bc0-a5b6-ac4e34252d3c",
    "path": "/home/user/music"
  }
}
```

**Text Summary**:
```
Downloaded Front cover (500) to cover.jpg (45231 bytes)
```

**Structured Data**:
```json
{
  "success": true,
  "file_path": "/home/user/music/cover.jpg",
  "file_size_bytes": 45231,
  "image_type": "Front",
  "thumbnail_size": "500",
  "source_url": "https://coverartarchive.org/release/..."
}
```

---

### Example 2: High-Resolution Cover with Custom Filename

Download original resolution cover with custom filename.

**Request**:
```json
{
  "name": "mb_cover_download",
  "arguments": {
    "mbid": "76df3287-6cda-33eb-8e9a-044b5e15ffdd",
    "path": "/home/user/music/nirvana",
    "filename": "in_utero_cover",
    "thumbnail_size": "original",
    "overwrite": true
  }
}
```

**Text Summary**:
```
Downloaded Front cover (original) to in_utero_cover.jpg (2847391 bytes)
```

**Structured Data**:
```json
{
  "success": true,
  "file_path": "/home/user/music/nirvana/in_utero_cover.jpg",
  "file_size_bytes": 2847391,
  "image_type": "Front",
  "thumbnail_size": "original",
  "source_url": "https://coverartarchive.org/release/..."
}
```

---

### Example 3: Small Thumbnail for Quick Preview

Download small 250px thumbnail.

**Request**:
```json
{
  "name": "mb_cover_download",
  "arguments": {
    "mbid": "65c70b9f-fdef-4bc0-a5b6-ac4e34252d3c",
    "path": "/tmp/previews",
    "filename": "preview",
    "thumbnail_size": "250"
  }
}
```

**Text Summary**:
```
Downloaded Front cover (250) to preview.jpg (8412 bytes)
```

---

## Image Selection Strategy

The tool uses an intelligent priority system to select the best available image:

### Priority Order

1. **Front Cover (Primary)**
   - Images marked with `front: true`
   - Images with type `"Front"` in the types array

2. **Fallback Images**
   - If no front cover available, selects first available image
   - Could be Back, Booklet, Medium, etc.

3. **Size Fallback Strategy**

When requested size is unavailable, the tool tries alternatives:

- **For 250px**: Try 250 → 500 → 1200 → original
- **For 500px**: Try 500 → 1200 → 250 → original
- **For 1200px**: Try 1200 → original
- **For original**: Always returns original (no fallback needed)

The `thumbnail_size` field in the result indicates the actual size downloaded.

---

## File Extension Detection

The tool automatically detects the correct file extension based on the image URL:
- `.jpg` or `.jpeg` for JPEG images
- `.png` for PNG images
- `.gif` for GIF images
- `.webp` for WebP images
- Default fallback: `.jpg`

---

## Error Handling

### Common Errors

**Invalid MBID Format**:
```json
{
  "isError": true,
  "content": "Invalid MBID format (expected UUID)"
}
```

**Release Not Found / No Cover Art**:
```json
{
  "isError": true,
  "content": "Failed to fetch cover art: Not found"
}
```

**Path Not a Directory**:
```json
{
  "isError": true,
  "content": "Path is not a directory: /path/to/file.txt"
}
```

**File Already Exists**:
```json
{
  "isError": true,
  "content": "File already exists: /path/to/cover.jpg. Use overwrite=true to replace"
}
```

**Path Security Violation**:
```json
{
  "isError": true,
  "content": "Path security validation failed: Path outside allowed root"
}
```

**No Images Available**:
```json
{
  "isError": true,
  "content": "No suitable image found: No images available"
}
```

---

## Security

### Path Validation

All file paths are validated against the configured root directory:
- Paths outside the root directory are rejected
- Path traversal attacks (`../`) are prevented
- Symlinks are validated (if enabled in configuration)

See [Path Security Documentation](../../reference/path-security.md) for details.

### Environment Configuration

Set the allowed root directory via environment variable:
```bash
export MCP_ROOT_PATH="/home/user/music"
```

---

## Rate Limiting

The Cover Art Archive API does not enforce strict rate limiting, but it's recommended to:
- Add delays between bulk downloads
- Respect the service and avoid excessive requests
- Cache downloaded covers to avoid redundant downloads

---

## API Source

This tool uses the [Cover Art Archive API](https://musicbrainz.org/doc/Cover_Art_Archive/API), which is:
- A joint project between the Internet Archive and MusicBrainz
- Free to use for all purposes
- Provides high-quality cover art for millions of releases
- Returns images in various sizes

---

## Related Tools

- **mb_release_search**: Find releases and get their MBIDs
- **mb_identify_record**: Identify audio files and get release information
- **write_metadata**: Embed cover art into audio file tags (future enhancement)

---

## Tips & Best Practices

1. **Finding MBIDs**: Use `mb_release_search` to find the MBID for a release
2. **Start Small**: Use 250px or 500px thumbnails for previews, save bandwidth
3. **High Quality**: Use `"original"` only when you need full resolution
4. **Batch Downloads**: Add 1-2 second delays between requests
5. **File Organization**: Use descriptive filenames (e.g., album name)
6. **Check Overwrite**: Set `overwrite: false` to avoid accidentally replacing files

---

## Example Workflow

```javascript
// 1. Search for a release
const searchResult = await mcp.call_tool("mb_release_search", {
  search_type: "release",
  query: "Nevermind Nirvana"
});

// 2. Extract MBID from results
const mbid = searchResult.releases[0].mbid;

// 3. Download cover art
const coverResult = await mcp.call_tool("mb_cover_download", {
  mbid: mbid,
  path: "/home/user/music/nirvana",
  filename: "nevermind_cover",
  thumbnail_size: "500"
});

// 4. Use the downloaded file
console.log(`Cover saved to: ${coverResult.file_path}`);
console.log(`Size: ${coverResult.file_size_bytes} bytes`);
```

---

## Troubleshooting

### Issue: "No suitable image found"

**Cause**: Release has no cover art in the archive

**Solution**:
- Verify the release exists on MusicBrainz.org
- Check if cover art is available on the release page
- Try a different release/edition of the album

### Issue: "Path security validation failed"

**Cause**: Target directory is outside allowed root

**Solution**:
- Set `MCP_ROOT_PATH` environment variable
- Use paths within the configured root directory
- Check directory permissions

### Issue: File extension is wrong

**Cause**: URL-based detection may not always be accurate

**Solution**:
- The tool prioritizes functionality over perfect extension detection
- Most music players/apps handle this gracefully
- File content is correct regardless of extension

---

## Technical Notes

- **Transport**: Supports both STDIO/TCP and HTTP transports
- **Blocking Operations**: Uses thread spawning to avoid runtime conflicts
- **Dependencies**: Uses `musicbrainz_rs` library with blocking reqwest
- **Image Formats**: Supports JPEG, PNG, GIF, WebP (auto-detected)
- **Thread Safety**: All operations are thread-safe and can run concurrently
