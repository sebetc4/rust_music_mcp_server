# mb_recording_search

Search for recordings (individual tracks) and find where they appear.

---

## Overview

The `mb_recording_search` tool allows you to:
- Search for recordings by name
- Look up recordings by MusicBrainz ID (MBID) with full details
- Find all releases containing a specific recording
- Discover live versions, covers, or remixes
- Track down rare or region-specific releases
- Returns structured JSON data with concise text summaries

**Output Format**: This tool follows MCP standards, returning a short text summary plus structured JSON data for programmatic access.

---

## Parameters

```typescript
{
  search_type: "recording" | "recording_releases",  // Type of search (required)
  query: string,                                     // Recording title or MBID (required)
  limit?: number                                     // Max results, 1-100 (default: 10)
}
```

### Parameter Details

- **search_type** (required)
  - `"recording"`: Search for recordings by title (or fetch by MBID with full details)
  - `"recording_releases"`: Find all releases containing a specific recording

- **query** (required)
  - Recording name for search (e.g., "Paranoid Android")
  - Or MBID for direct lookup (e.g., "6bf6f137-f7e5-4e40-880f-db35b3f9c272")
  - Tool automatically detects MBID format and returns full details

- **limit** (optional)
  - Range: 1-100
  - Default: 10
  - Applies to search results

---

## Output Format

The tool returns:
1. **Text Summary**: A concise one-line description of results
2. **Structured JSON Data**: Complete data in a standardized format for programmatic access

This follows MCP standards for structured tool output, providing both human-readable summaries and machine-parseable data.

---

## Examples

### Example 1: Search Recording by Name

Basic recording search by title.

**Request**:
```json
{
  "name": "mb_recording_search",
  "arguments": {
    "search_type": "recording",
    "query": "Paranoid Android"
  }
}
```

**Text Summary**:
```
Found 5 recording(s) matching 'Paranoid Android'
```

**Structured Data**:
```json
{
  "recordings": [
    {
      "title": "Paranoid Android",
      "mbid": "6bf6f137-f7e5-4e40-880f-db35b3f9c272",
      "artist": "Radiohead",
      "duration": "6:23",
      "disambiguation": null
    }
  ],
  "total_count": 5,
  "query": "Paranoid Android"
}
```

---

### Example 2: Get Full Recording Details by MBID

When you provide an MBID, the tool returns comprehensive details including releases and genres.

**Request**:
```json
{
  "name": "mb_recording_search",
  "arguments": {
    "search_type": "recording",
    "query": "6bf6f137-f7e5-4e40-880f-db35b3f9c272"
  }
}
```

**Text Summary**:
```
'Paranoid Android' by Radiohead (6:23) - found on 12 release(s)
```

**Structured Data**:
```json
{
  "title": "Paranoid Android",
  "mbid": "6bf6f137-f7e5-4e40-880f-db35b3f9c272",
  "artist": "Radiohead",
  "duration": "6:23",
  "disambiguation": null,
  "artist_mbids": [
    {
      "name": "Radiohead",
      "mbid": "a74b1b7f-71a5-4011-9441-d0b5e4122711"
    }
  ],
  "releases": [
    {
      "title": "OK Computer",
      "mbid": "52709206-8816-3c12-9ff6-f957f2f1eecf",
      "country": "GB",
      "year": "1997"
    }
  ],
  "genres": ["alternative rock", "art rock"]
}
```

---

### Example 3: Find Where Recording Appears

Discover all releases containing a specific track.

**Request**:
```json
{
  "name": "mb_recording_search",
  "arguments": {
    "search_type": "recording_releases",
    "query": "6bf6f137-f7e5-4e40-880f-db35b3f9c272",
    "limit": 10
  }
}
```

**Text Summary**:
```
'Paranoid Android' by Radiohead - found on 10 release(s)
```

**Structured Data**:
```json
{
  "recording_title": "Paranoid Android",
  "recording_mbid": "6bf6f137-f7e5-4e40-880f-db35b3f9c272",
  "recording_artist": "Radiohead",
  "duration": "6:23",
  "releases": [
    {
      "title": "OK Computer",
      "mbid": "52709206-8816-3c12-9ff6-f957f2f1eecf",
      "artist": "Radiohead",
      "date": "1997",
      "country": "GB"
    },
    {
      "title": "OK Computer OKNOTOK 1997 2017",
      "mbid": "ab96639e-b9e9-481c-abd6-8abd92e85661",
      "artist": "Radiohead",
      "date": "2017",
      "country": "XW"
    }
  ],
  "total_count": 10
}
```

---

## Use Cases

### 1. Identify Tracks Across Different Releases
Find the same recording in albums, compilations, and singles:
```json
{
  "search_type": "recording_releases",
  "query": "recording-mbid-here",
  "limit": 50
}
```

### 2. Find Live Versions, Covers, or Remixes
Search for variations of a track:
```json
{
  "search_type": "recording",
  "query": "One (live)"
}
```

### 3. Discover Compilation Albums
Track which compilations contain a specific song:
```json
{
  "search_type": "recording_releases",
  "query": "recording-mbid-here"
}
```

### 4. Track Down Rare or Region-Specific Releases
Find where a recording was released:
```json
{
  "search_type": "recording_releases",
  "query": "B-side track name"
}
```

### 5. Verify Recording Information
Confirm track details before tagging:
```json
{
  "search_type": "recording",
  "query": "Track Name"
}
```

### 6. Get Complete Recording Metadata
Fetch full details including artist MBIDs and genres:
```json
{
  "search_type": "recording",
  "query": "recording-mbid-here"
}
```

---

## Response Fields

### Recording Search by Title (`search_type: "recording"` with title)

Returns `RecordingSearchResult` with:

```typescript
{
  recordings: [
    {
      title: string,              // Recording name
      mbid: string,               // Unique MusicBrainz recording identifier
      artist: string,             // Primary artist name(s)
      duration: string | null,    // Track length (MM:SS)
      disambiguation: string | null  // Additional context
    }
  ],
  total_count: number,            // Number of recordings returned
  query: string                   // Original search query
}
```

### Recording Details by MBID (`search_type: "recording"` with MBID)

Returns `RecordingDetails` with:

```typescript
{
  title: string,                  // Recording name
  mbid: string,                   // Recording MBID
  artist: string,                 // Primary artist name
  duration: string | null,        // Track length (MM:SS)
  disambiguation: string | null,  // Additional context
  artist_mbids: [
    {
      name: string,               // Artist name
      mbid: string                // Artist MBID
    }
  ],
  releases: [
    {
      title: string,              // Release title
      mbid: string,               // Release MBID
      country: string | null,     // ISO country code
      year: string | null         // Release year
    }
  ],
  genres: string[]                // Genre tags
}
```

### Recording Releases (`search_type: "recording_releases"`)

Returns `RecordingReleasesResult` with:

```typescript
{
  recording_title: string,        // Recording name
  recording_mbid: string,         // Recording MBID
  recording_artist: string,       // Recording artist
  duration: string | null,        // Track length (MM:SS)
  releases: [
    {
      title: string,              // Release title
      mbid: string,               // Release MBID
      artist: string,             // Release artist (may differ for compilations)
      date: string | null,        // Release year
      country: string | null      // ISO country code
    }
  ],
  total_count: number             // Number of releases returned
}
```

---

## Understanding Recordings vs Tracks

In MusicBrainz terminology:
- **Recording**: The unique performance/version of a song
- **Track**: A recording's appearance on a specific release

**Example**:
- Recording: "Bohemian Rhapsody" (studio version)
- Tracks: Appears on "A Night at the Opera", "Greatest Hits", "Bohemian Rhapsody (single)", etc.

One recording can have many tracks across different releases.

---

## Related Tools

- [mb_release_search](mb_release_search.md) - Get details about releases containing the recording
- [mb_artist_search](mb_artist_search.md) - Find artist information
- [mb_identify_record](mb_identify_record.md) - Identify unknown audio files

---

## Common Patterns

### Pattern 1: Track Identification to Release Info
```
1. mb_recording_search (search_type: "recording", query: "name") → Get recording MBID
2. mb_recording_search (search_type: "recording_releases", query: mbid) → Get releases
3. mb_release_search (search_type: "release_recordings", query: release_mbid) → Get full release details
```

### Pattern 2: Compilation Discovery
```
1. mb_recording_search (search_type: "recording_releases", query: mbid, limit: 100)
2. Parse structured data to filter compilations
3. Get compilation release details
```

### Pattern 3: Live/Alternative Version Discovery
```
1. Search base recording: mb_recording_search (search_type: "recording", query: "Song Name")
2. Search variations: mb_recording_search (search_type: "recording", query: "Song Name (live)")
3. Compare durations and appearances from structured data
```

---

## Tips

### Get Full Recording Details
Use MBID queries to get comprehensive information:
```json
{
  "search_type": "recording",
  "query": "recording-mbid-here"
}
```
This returns artist MBIDs, releases, and genres.

### Find Rare B-Sides
Use recording_releases to locate singles and EPs:
```json
{
  "search_type": "recording_releases",
  "query": "recording-mbid-here"
}
```

### Verify Audio File Identity
Compare durations to ensure correct match:
```json
{
  "search_type": "recording",
  "query": "Track Name"
}
```
Check that the returned duration in structured data matches your file.

---

## See Also

- [Common Concepts](common-concepts.md) - MBIDs, recordings vs tracks
- [Rate Limiting](rate-limiting.md) - API usage guidelines
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
