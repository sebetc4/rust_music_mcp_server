# mb_release_search

Search for releases (albums, singles, etc.), retrieve tracklists, and find all versions of a release group.

---

## Overview

The `mb_release_search` tool allows you to:
- Search for releases by title
- Get complete tracklists with recording MBIDs
- Find all versions of a release group (remasters, editions, countries)
- Access detailed release information with structured JSON data
- Returns concise text summaries with structured output

**Output Format**: This tool follows MCP standards, returning a short text summary plus structured JSON data for programmatic access.

---

## Parameters

```typescript
{
  search_type: "release" | "release_recordings" | "release_group_releases",  // Type of search (required)
  query: string,                                                              // Release title or MBID (required)
  limit?: number                                                              // Max results, 1-100 (default: 10)
}
```

### Parameter Details

- **search_type** (required)
  - `"release"`: Search for releases by title
  - `"release_recordings"`: Get all tracks/recordings in a release
  - `"release_group_releases"`: Get all versions of a release group

- **query** (required)
  - Release or release group title for search (e.g., "OK Computer")
  - Or MBID for direct lookup (e.g., "52709206-8816-3c12-9ff6-f957f2f1eecf")
  - Tool automatically detects MBID format

- **limit** (optional)
  - Range: 1-100
  - Default: 10
  - Applies to search results

---

## Output Format

The tool returns:
1. **Text Summary**: A concise one-line description of results (e.g., "Found 10 release(s) matching 'OK Computer'")
2. **Structured JSON Data**: Complete data in a standardized format for programmatic access

This follows MCP standards for structured tool output, providing both human-readable summaries and machine-parseable data.

---

## Examples

### Example 1: Search Release by Name

Basic release search by title.

**Request**:
```json
{
  "name": "mb_release_search",
  "arguments": {
    "search_type": "release",
    "query": "OK Computer"
  }
}
```

**Text Summary**:
```
Found 10 release(s) matching 'OK Computer'
```

**Structured Data**:
```json
{
  "releases": [
    {
      "title": "OK Computer",
      "mbid": "52709206-8816-3c12-9ff6-f957f2f1eecf",
      "artist": "Radiohead",
      "year": "1997",
      "country": "GB",
      "barcode": "724384260927"
    }
  ],
  "total_count": 10,
  "query": "OK Computer"
}
```

---

### Example 2: Get Release with Tracklist

Retrieve complete track information including durations and MBIDs.

**Request**:
```json
{
  "name": "mb_release_search",
  "arguments": {
    "search_type": "release_recordings",
    "query": "52709206-8816-3c12-9ff6-f957f2f1eecf"
  }
}
```

**Text Summary**:
```
Track listing for 'OK Computer' by Radiohead (12 track(s))
```

**Structured Data**:
```json
{
  "release_title": "OK Computer",
  "release_mbid": "52709206-8816-3c12-9ff6-f957f2f1eecf",
  "artist": "Radiohead",
  "media": [
    {
      "disc_number": 1,
      "disc_title": null,
      "tracks": [
        {
          "position": 1,
          "title": "Airbag",
          "duration": "4:44",
          "recording_mbid": "d4f52c25-e80e-4839-9484-8ce5a1c54d89",
          "artist": null
        },
        {
          "position": 2,
          "title": "Paranoid Android",
          "duration": "6:23",
          "recording_mbid": "6bf6f137-f7e5-4e40-880f-db35b3f9c272",
          "artist": null
        }
      ]
    }
  ],
  "total_tracks": 12
}
```

---

### Example 3: Find Release Group Versions

Discover all editions of a release (remasters, different countries, formats).

**Request**:
```json
{
  "name": "mb_release_search",
  "arguments": {
    "search_type": "release_group_releases",
    "query": "Discovery",
    "limit": 5
  }
}
```

**Text Summary**:
```
Found 5 version(s) of 'Discovery' by Daft Punk
```

**Structured Data**:
```json
{
  "release_group_title": "Discovery",
  "release_group_mbid": "b81bcdb6-4223-43e9-a6a3-90537f8c0eb5",
  "artist": "Daft Punk",
  "releases": [
    {
      "title": "Discovery",
      "mbid": "...",
      "date": "2001-03-07",
      "country": "FR"
    },
    {
      "title": "Discovery",
      "mbid": "...",
      "date": "2001-03-13",
      "country": "US"
    }
  ],
  "total_count": 5
}
```

---

## Use Cases

### 1. Find Official Release Information
Get accurate release details for metadata tagging:
```json
{
  "search_type": "release",
  "query": "The Dark Side of the Moon"
}
```

### 2. Get Complete Tracklists
Extract all tracks with MBIDs for a release:
```json
{
  "search_type": "release_recordings",
  "query": "52709206-8816-3c12-9ff6-f957f2f1eecf"
}
```

### 3. Compare Different Editions
Find all versions to choose the right one:
```json
{
  "search_type": "release_group_releases",
  "query": "Abbey Road",
  "limit": 50
}
```

### 4. Verify Release Dates and Countries
Confirm which edition you have:
```json
{
  "search_type": "release_group_releases",
  "query": "Random Access Memories"
}
```

### 5. Extract Track MBIDs for Processing
Get recording identifiers for further operations:
```json
{
  "search_type": "release_recordings",
  "query": "release-mbid-here"
}
```

---

## Response Fields

### Release Search (`search_type: "release"`)

Returns `ReleaseSearchResult` with:

```typescript
{
  releases: [
    {
      title: string,              // Release name
      mbid: string,               // Unique MusicBrainz release identifier
      artist: string,             // Primary artist name(s)
      year: string | null,        // Release year (e.g., "1997")
      country: string | null,     // ISO country code
      barcode: string | null      // Barcode/UPC if available
    }
  ],
  total_count: number,            // Number of releases returned
  query: string                   // Original search query
}
```

### Release Recordings (`search_type: "release_recordings"`)

Returns `ReleaseRecordingsResult` with:

```typescript
{
  release_title: string,          // Release name
  release_mbid: string,           // Release MBID
  artist: string,                 // Primary artist
  media: [
    {
      disc_number: number,        // Disc number (1-based)
      disc_title: string | null,  // Disc title if multi-disc
      tracks: [
        {
          position: number,       // Track number
          title: string,          // Track name
          duration: string | null,// Track length (MM:SS)
          recording_mbid: string, // Unique recording identifier
          artist: string | null   // Track artist (if different from release artist)
        }
      ]
    }
  ],
  total_tracks: number            // Total number of tracks
}
```

### Release Group Releases (`search_type: "release_group_releases"`)

Returns `ReleaseGroupReleasesResult` with:

```typescript
{
  release_group_title: string,    // Release group title
  release_group_mbid: string,     // Release group MBID
  artist: string,                 // Artist name
  releases: [
    {
      title: string,              // Release title
      mbid: string,               // Release MBID
      date: string | null,        // Release date (YYYY-MM-DD or YYYY)
      country: string | null      // ISO country code
    }
  ],
  total_count: number             // Number of versions returned
}
```

---

## Release Status Types

| Status | Description |
|--------|-------------|
| **Official** | Legitimate, authorized release |
| **Promotion** | Promotional release (radio, press) |
| **Bootleg** | Unauthorized, unofficial release |
| **Pseudo-Release** | Non-physical (digital, streaming) |

---

## Release Types

| Type | Description |
|------|-------------|
| **Album** | Standard full-length album |
| **Single** | Single track release |
| **EP** | Extended play (3-6 tracks) |
| **Compilation** | Tracks from various sources |
| **Soundtrack** | Music from film/TV/game |
| **Live** | Recorded at live performance |
| **Remix** | Remixed versions of tracks |
| **Broadcast** | Radio/TV broadcast |

---

## Related Tools

- [mb_artist_search](mb_artist_search.md) - Find artist to get their releases
- [mb_recording_search](mb_recording_search.md) - Search for specific tracks
- [mb_identify_record](mb_identify_record.md) - Identify audio files to get release info

---

## Common Patterns

### Pattern 1: Complete Album Metadata Extraction
```
1. mb_release_search (search_type: "release", query: "name") → Get MBID
2. mb_release_search (search_type: "release_recordings", query: mbid) → Get full tracklist
3. Use recording MBIDs for individual track processing
```

### Pattern 2: Find Best Release Version
```
1. mb_release_search (search_type: "release_group_releases", query: "name")
2. Compare dates, countries from structured data
3. Select preferred version's MBID
4. mb_release_search (search_type: "release_recordings", query: mbid) → Get final data
```

### Pattern 3: Bulk Tracklist Export
```
1. mb_artist_search (search_type: "artist_releases", query: "name") → Get all release MBIDs
2. For each release MBID:
   mb_release_search (search_type: "release_recordings", query: mbid)
```

---

## See Also

- [Common Concepts](common-concepts.md) - MBIDs, release types, status codes
- [Rate Limiting](rate-limiting.md) - API usage guidelines
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
