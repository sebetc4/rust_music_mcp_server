# mb_identify_record

Identify audio files using acoustic fingerprinting via AcoustID and enrich with MusicBrainz metadata.

---

## Overview

The `mb_identify_record` tool allows you to:
- Identify unknown music files by their acoustic signature
- Auto-tag music libraries with accurate metadata
- Verify file integrity against expected tracks
- Discover official release information for any audio file
- Get complete MusicBrainz data for identified recordings
- Returns structured JSON data with concise text summaries

**Output Format**: This tool follows MCP standards, returning a short text summary plus structured JSON data for programmatic access.

This tool combines:
- **Chromaprint** (fpcalc) for acoustic fingerprinting
- **AcoustID** for fingerprint matching
- **MusicBrainz** for detailed metadata

---

## Parameters

```typescript
{
  file_path: string,                         // Path to audio file (required)
  limit?: number,                            // Max results (default: 3, max: 10)
  metadata_level?: "minimal" | "basic" | "full"  // Level of detail (default: "basic")
}
```

### Parameter Details

- **file_path** (required)
  - Absolute or relative path to audio file
  - File must exist and be readable
  - Supports various audio formats (see [Supported Formats](#supported-formats))

- **limit** (optional)
  - Range: 1-10
  - Default: 3
  - Number of identification matches to return

- **metadata_level** (optional)
  - `minimal`: Only MusicBrainz Recording IDs (fastest)
  - `basic` (default): Recording IDs + title, artists, and duration
  - `full`: Complete metadata including release groups, albums, and dates

---

## Output Format

The tool returns:
1. **Text Summary**: A concise description of the identification result
2. **Structured JSON Data**: Complete fingerprint match data in a standardized format

This follows MCP standards for structured tool output, providing both human-readable summaries and machine-parseable data.

---

## Metadata Levels

### Minimal
**What's included**: Only MusicBrainz Recording IDs

**Use case**: Quick identification, batch processing when you only need IDs

**Structured Output**: Recording IDs without titles or artist names

---

### Basic (Default)
**What's included**: Recording IDs + title, artist names, and duration

**Use case**: Standard music library tagging (recommended for most cases)

**Structured Output**: Recording information with basic metadata

---

### Full
**What's included**: Complete metadata including release groups, albums, formats, and dates

**Use case**: Complete metadata collection, detailed music databases

**Structured Output**: Comprehensive recording and release information

---

## Requirements

### System Requirements

1. **fpcalc binary** must be installed
   - Part of the Chromaprint package
   - Used to generate acoustic fingerprints
   - Must be in system PATH or specified location

2. **AcoustID API Key** (Configured via Environment Variable)
   - A default public API key is provided for immediate out-of-the-box use
   - Works with limited rate limits (shared across all default key users)
   - **For production use**, set your own key via environment variable: `MCP_ACOUSTID_API_KEY`
   - Free API keys available at: https://acoustid.org/api-key

   **Configuration**:
   ```bash
   # Set in .env file (recommended for development)
   MCP_ACOUSTID_API_KEY=your_personal_key_here

   # Or set as system environment variable (recommended for production)
   export MCP_ACOUSTID_API_KEY=your_personal_key_here
   ```

   See [Configuration Guide](../../guides/configuration.md) for detailed setup instructions.

### Installation Instructions

#### Ubuntu/Debian
```bash
sudo apt-get install libchromaprint-tools
```

#### macOS
```bash
brew install chromaprint
```

#### Windows
Download from: https://acoustid.org/chromaprint

#### Verify Installation
```bash
fpcalc -version
```

---

## Supported Formats

The tool supports any audio format that Chromaprint can process:

- **MP3** (.mp3)
- **FLAC** (.flac)
- **M4A/AAC** (.m4a, .aac)
- **WAV** (.wav)
- **OGG Vorbis** (.ogg)
- **WMA** (.wma)
- **Opus** (.opus)
- **APE** (.ape)
- **WavPack** (.wv)

Most common audio formats are supported.

---

## Examples

### Example 1: Basic Identification (Default)

Identify an unknown file with default settings.

**Request**:
```json
{
  "name": "mb_identify_record",
  "arguments": {
    "file_path": "/music/unknown_track.mp3"
  }
}
```

**Text Summary**:
```
Identified: 'Paranoid Android' by Radiohead (95% confidence, 3 match(es))
```

**Structured Data**:
```json
{
  "file": "/music/unknown_track.mp3",
  "metadata_level": "basic",
  "matches": [
    {
      "rank": 1,
      "confidence": 0.95,
      "acoustid": "12345678-1234-1234-1234-123456789012",
      "recordings": [
        {
          "id": "6bf6f137-f7e5-4e40-880f-db35b3f9c272",
          "title": "Paranoid Android",
          "duration": 383,
          "artists": ["Radiohead"],
          "release_groups": null
        }
      ]
    },
    {
      "rank": 2,
      "confidence": 0.87,
      "acoustid": "87654321-4321-4321-4321-210987654321",
      "recordings": [
        {
          "id": "another-recording-mbid",
          "title": "Paranoid Android",
          "duration": 383,
          "artists": ["Radiohead"],
          "release_groups": null
        }
      ]
    }
  ],
  "status": "success"
}
```

---

### Example 2: Minimal Metadata for Batch Processing

Fast identification with minimal data transfer.

**Request**:
```json
{
  "name": "mb_identify_record",
  "arguments": {
    "file_path": "/music/batch/track_001.mp3",
    "metadata_level": "minimal",
    "limit": 3
  }
}
```

**Text Summary**:
```
Identified audio: 3 match(es) found (best: 95% confidence, Recording ID: 6bf6f137-f7e5-4e40-880f-db35b3f9c272)
```

**Structured Data**:
```json
{
  "file": "/music/batch/track_001.mp3",
  "metadata_level": "minimal",
  "matches": [
    {
      "rank": 1,
      "confidence": 0.95,
      "acoustid": "12345678-1234-1234-1234-123456789012",
      "recordings": [
        {
          "id": "6bf6f137-f7e5-4e40-880f-db35b3f9c272",
          "title": null,
          "duration": null,
          "artists": null,
          "release_groups": null
        }
      ]
    }
  ],
  "status": "success"
}
```

**Use case**: Processing hundreds/thousands of files quickly.

---

### Example 3: Full Metadata for Complete Tagging

Get all available information for comprehensive tagging.

**Request**:
```json
{
  "name": "mb_identify_record",
  "arguments": {
    "file_path": "/music/discovery_01.flac",
    "metadata_level": "full",
    "limit": 3
  }
}
```

**Text Summary**:
```
Identified: 'Paranoid Android' by Radiohead (95% confidence, 2 release group(s), 3 total match(es))
```

**Structured Data**:
```json
{
  "file": "/music/discovery_01.flac",
  "metadata_level": "full",
  "matches": [
    {
      "rank": 1,
      "confidence": 0.95,
      "acoustid": "12345678-1234-1234-1234-123456789012",
      "recordings": [
        {
          "id": "6bf6f137-f7e5-4e40-880f-db35b3f9c272",
          "title": "Paranoid Android",
          "duration": 383,
          "artists": ["Radiohead"],
          "release_groups": [
            {
              "name": "OK Computer",
              "type": "Album"
            },
            {
              "name": "OK Computer OKNOTOK 1997 2017",
              "type": "Album"
            }
          ]
        }
      ]
    }
  ],
  "status": "success"
}
```

**Response includes**:
- Complete track information
- All release groups containing the recording
- Album types and additional metadata

---

### Example 4: Verify File Identity

Check if a file matches expected content.

**Request**:
```json
{
  "name": "mb_identify_record",
  "arguments": {
    "file_path": "/music/radiohead/ok_computer/02_paranoid_android.mp3"
  }
}
```

Compare the returned recording MBID and metadata against expected values.

---

## Understanding Results

### Match Scores

AcoustID returns a confidence score (0.0 to 1.0):

- **0.95 - 1.0**: Excellent match (almost certainly correct)
- **0.85 - 0.95**: Very good match (very likely correct)
- **0.70 - 0.85**: Good match (probably correct, verify manually)
- **< 0.70**: Uncertain match (requires manual verification)

**Recommendation**: For automated tagging, use matches with score ≥ 0.90.

---

### Multiple Matches

The tool may return multiple matches:

1. **Same recording, different releases**
   - Most common scenario
   - Different countries, formats, or editions
   - All matches are valid

2. **Similar recordings**
   - Live versions vs studio versions
   - Original vs remaster
   - Different performances

3. **Ambiguous matches**
   - Low confidence scores
   - Requires manual verification

Always check the score and compare metadata against known information.

---

## Error Handling

### Error: Invalid or Expired API Key

```
Error: AcoustID API key is invalid or expired.

The default public key is no longer valid or has exceeded its rate limits.
Please set your own API key via environment variable: MCP_ACOUSTID_API_KEY
You can request a free API key at: https://acoustid.org/api-key
```

**What happened**:
The tool uses a default public API key for out-of-the-box functionality. This key may:
- Be rate-limited due to high usage
- Have expired or become invalid
- Be blocked temporarily

**Solutions**:
1. **Get your own free API key** (recommended):
   ```bash
   # Visit https://acoustid.org/api-key to get your key
   export MCP_ACOUSTID_API_KEY="your_key_here"
   ```

2. **Wait and retry**: If using the default key, rate limits may reset after a few minutes

3. **Check AcoustID service status**: Visit https://acoustid.org/ to verify the service is operational

**Why this happens immediately**:
The tool detects invalid API keys on the first request and fails fast without retrying, saving you time and network bandwidth.

---

### Error: fpcalc Binary Not Found

```
Error: fpcalc binary not found

The fpcalc tool is required for audio fingerprinting.

Install instructions:
  - Ubuntu/Debian: sudo apt-get install libchromaprint-tools
  - macOS: brew install chromaprint
  - Windows: Download from https://acoustid.org/chromaprint
```

**Solution**: Install chromaprint package for your system.

---

### Error: File Not Found

```
Error: File not found: /music/missing.mp3

Please check:
  - File path is correct
  - File exists and is accessible
  - You have read permissions
```

**Solutions**:
- Verify file path (absolute vs relative)
- Check file exists: `ls -l /music/missing.mp3`
- Check permissions: `chmod +r /music/missing.mp3`

---

### Error: Unsupported Format

```
Error: Failed to generate fingerprint for /music/file.xyz

Possible causes:
  - Unsupported audio format
  - Corrupted audio file
  - File is not an audio file
```

**Solutions**:
- Convert to supported format (MP3, FLAC, etc.)
- Test file with audio player
- Check file isn't corrupted

---

### No Matches Found

```
Audio Identification Results
============================

File: /music/rare_bootleg.mp3
Fingerprint generated successfully (duration: 180.2s)

No matches found in AcoustID database.

This could mean:
  - The recording is not in MusicBrainz
  - The audio quality is too poor for fingerprinting
  - The file is heavily edited or remixed
  - This is a rare or unreleased recording
```

**Possible causes**:
1. **Not in database**: Very rare, unreleased, or new recordings
2. **Poor quality**: Low bitrate, heavy compression, or damaged files
3. **Heavy editing**: Speed changes, pitch shifts, or extensive remixing
4. **Non-music audio**: Podcasts, audiobooks, sound effects

**Solutions**:
- Try a different recording of the same track
- Manually search MusicBrainz for the recording
- Add the recording to MusicBrainz (if appropriate)

---

## Use Cases

### 1. Identify Unknown Music Files
**Scenario**: Downloaded files with poor/missing tags
```json
{
  "file_path": "/downloads/unknown_001.mp3",
  "metadata_level": "basic"
}
```

### 2. Auto-Tag Music Libraries
**Scenario**: Clean up entire music collection
```
For each file in library:
  mb_identify_record (file_path: ..., metadata_level: "basic")
  write_metadata (using returned data)
```

### 3. Verify File Integrity
**Scenario**: Ensure files match expected content
```json
{
  "file_path": "/music/albums/radiohead/ok_computer/02.mp3"
}
```
Compare returned MBID against expected recording MBID.

### 4. Discover Official Release Information
**Scenario**: Bootleg or ripped CD with no metadata
```json
{
  "file_path": "/bootlegs/unknown_concert.flac",
  "metadata_level": "full"
}
```

### 5. Clean Up Poorly-Tagged Collections
**Scenario**: Fix incorrect or inconsistent tags
```
For files with suspicious metadata:
  mb_identify_record (file_path: ..., metadata_level: "full")
  Compare against existing tags
  Update if identification is confident
```

---

## Performance Considerations

### Fingerprint Generation Time

| File Duration | Typical Processing Time |
|---------------|------------------------|
| 3 minutes | 1-3 seconds |
| 5 minutes | 2-5 seconds |
| 10 minutes | 4-8 seconds |
| 60 minutes | 20-40 seconds |

Time varies based on:
- File format and compression
- System CPU speed
- Disk I/O speed

---

### Metadata Level Impact

| Level | API Calls | Typical Response Time |
|-------|-----------|----------------------|
| Minimal | 1 | Fast (~1s) |
| Basic | 1-2 | Medium (~2-3s) |
| Full | 3-5 | Slower (~5-8s) |

Full metadata requires multiple MusicBrainz API calls.

---

### Batch Processing Tips

1. **Use minimal metadata level** for initial pass
2. **Add delays** between requests (respect rate limits)
3. **Cache results** to avoid re-processing
4. **Process in parallel** (but respect rate limits)
5. **Use basic level** for most tagging needs (good balance)
6. **Get your own API key** for batch processing to avoid rate limits on the default public key

---

## Related Tools

- [mb_recording_search](mb_recording_search.md) - Search by name when identification fails
- [mb_release_search](mb_release_search.md) - Get additional release information
- [write_metadata](../metadata-tools.md) - Apply identified metadata to files

# Metadata Levels for Audio Identification

The `mb_identify_record` tool supports three metadata levels when querying the AcoustID API. This document explains the differences and when to use each level.

## Overview

When identifying audio files via acoustic fingerprinting, you can choose how much metadata to retrieve from the AcoustID/MusicBrainz database. More metadata means more API data transfer and slightly longer response times, but provides richer information.

## Available Levels

### Minimal (`"minimal"`)

**Returns:** Only MusicBrainz Recording IDs

**Use case:** When you only need the Recording ID to perform additional lookups later, or when you want the fastest possible response.

**Example response:**
```
Match #1 (Confidence: 95%)
AcoustID: 12345678-1234-1234-1234-123456789012

MusicBrainz Recording(s):
  Recording #1
  ID: abcd1234-5678-90ab-cdef-1234567890ab
```

**JSON Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "mb_identify_record",
    "arguments": {
      "file_path": "/path/to/audio.mp3",
      "metadata_level": "minimal"
    }
  },
  "id": 1
}
```

### Basic (`"basic"`) - **DEFAULT**

**Returns:** Recording IDs + track title, artist names, and duration

**Use case:** When you need basic identification information without overwhelming detail. Good for most common use cases.

**Example response:**
```
Match #1 (Confidence: 95%)
AcoustID: 12345678-1234-1234-1234-123456789012

MusicBrainz Recording(s):
  Recording #1
  ID: abcd1234-5678-90ab-cdef-1234567890ab
  Title: Dubwise Attraction
  Duration: 5:32
  Artist(s): Panda Dub
```

**JSON Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "mb_identify_record",
    "arguments": {
      "file_path": "/path/to/audio.mp3",
      "metadata_level": "basic"
    }
  },
  "id": 1
}
```

**Note:** This is the default level if `metadata_level` is not specified.

### Full (`"full"`)

**Returns:** Recording IDs + complete metadata including releases, countries, dates, track counts, and more

**Use case:** When you need comprehensive information about the recording, including all associated releases.

**Example response:**
```
Match #1 (Confidence: 95%)
AcoustID: 12345678-1234-1234-1234-123456789012

MusicBrainz Recording(s):
  Recording #1
  ID: abcd1234-5678-90ab-cdef-1234567890ab
  Title: Dubwise Attraction
  Duration: 5:32
  Artist(s): Panda Dub

  Releases:
    1. Antilogy
       Country: FR | Date: 2012-11-19
       Tracks: 14
    2. Antilogy (Digital)
       Country: XW | Date: 2012-11-19
       Tracks: 14
    ... and 3 more release(s)
```

**JSON Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "mb_identify_record",
    "arguments": {
      "file_path": "/path/to/audio.mp3",
      "metadata_level": "full",
      "limit": 3
    }
  },
  "id": 1
}
```

## Performance Considerations

| Level    | API Response Size | Typical Response Time | Best For                          |
|----------|-------------------|----------------------|-----------------------------------|
| Minimal  | ~500 bytes        | Fastest              | Bulk identification, ID-only needs |
| Basic    | ~1-2 KB           | Fast                 | General purpose identification    |
| Full     | ~5-10 KB          | Slower               | Detailed music library management |

## Implementation Details

Under the hood, the metadata levels map to AcoustID API parameters:

- **Minimal**: `meta=recordingids`
- **Basic**: `meta=recordings`
- **Full**: `meta=recordings releasegroups compress`

See [identify_record.rs:52-65](../../../src/domains/tools/definitions/mb/identify_record.rs#L52-L65) for the implementation.

### API Key Handling

The tool implements intelligent API key error handling:

1. **Default Key**: A public API key is provided by default for immediate functionality
2. **Fail-Fast Detection**: Invalid/expired API keys are detected on the first request
3. **No Retries on Auth Errors**: HTTP 400/401/403 responses immediately return an `InvalidApiKey` error
4. **Clear Error Messages**: Users receive actionable guidance on how to set their own API key

This approach balances:
- **Ease of use**: Works out-of-the-box without configuration
- **Performance**: No wasted retry attempts on authentication failures
- **Scalability**: Clear path to production use with personal API keys

See [identify_record.rs:459-469](../../../src/domains/tools/definitions/mb/identify_record.rs#L459-L469) for the retry logic and [identify_record.rs:526-535](../../../src/domains/tools/definitions/mb/identify_record.rs#L526-L535) for API key error detection.

## Schema Definition

The `metadata_level` field accepts a string with one of three values:

```typescript
type MetadataLevel = "minimal" | "basic" | "full"
```

**JSON Schema:**
```json
{
  "metadata_level": {
    "oneOf": [
      {
        "const": "minimal",
        "description": "Only MusicBrainz recording IDs (minimal, fastest)",
        "type": "string"
      },
      {
        "const": "basic",
        "description": "MusicBrainz recording IDs + basic track info (title, artists)",
        "type": "string"
      },
      {
        "const": "full",
        "description": "Full metadata including releases, labels, and more",
        "type": "string"
      }
    ]
  }
}
```

## Testing with Insomnia

When using Insomnia to test the HTTP transport:

1. Create a POST request to `http://localhost:4000/mcp`
2. Set `Content-Type: application/json`
3. Use the following body template:

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "mb_identify_record",
    "arguments": {
      "file_path": "/path/to/your/audio.mp3",
      "metadata_level": "basic",
      "limit": 3
    }
  },
  "id": 1
}
```

4. Change `metadata_level` to `"minimal"`, `"basic"`, or `"full"` as needed

## Troubleshooting

### Issue: Field not appearing in Insomnia autocomplete

**Fixed in:** Latest version with `#[schemars(rename_all = "lowercase")]` attribute

**Solution:** Ensure you're using the latest version of the server that includes the schema fix.

### Issue: Invalid value error

**Cause:** Using uppercase values like `"Minimal"` or `"BASIC"`

**Solution:** Always use lowercase values: `"minimal"`, `"basic"`, `"full"`

## See Also

- [Audio Fingerprinting Guide](./AUDIO_FINGERPRINTING.md)
- [MusicBrainz Integration](./MUSICBRAINZ.md)
- [Tool Implementation](../src/domains/tools/definitions/mb/identify_record.rs)
---

## External Resources

- **AcoustID Website**: https://acoustid.org/
- **Chromaprint GitHub**: https://github.com/acoustid/chromaprint
- **MusicBrainz Picard**: https://picard.musicbrainz.org/ (GUI tool using similar technology)
- **AcoustID API Docs**: https://acoustid.org/webservice

---

## See Also

- [Common Concepts](common-concepts.md) - MBIDs, recordings, releases
- [Rate Limiting](rate-limiting.md) - API usage for batch processing
- [Troubleshooting](troubleshooting.md) - Common identification issues
