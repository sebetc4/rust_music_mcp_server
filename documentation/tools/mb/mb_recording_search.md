# mb_recording_search

Search for recordings (individual tracks) and find where they appear.

---

## Overview

The `mb_recording_search` tool allows you to:
- Search for recordings by name and artist
- Look up recordings by MusicBrainz ID (MBID)
- Find all releases containing a specific recording
- Discover live versions, covers, or remixes
- Track down rare or region-specific releases

---

## Parameters

```typescript
{
  recording?: string,                // Recording name to search
  recording_mbid?: string,           // Recording MBID for direct lookup
  artist?: string,                   // Filter by artist name
  find_appearances?: boolean,        // Find releases containing this track (default: false)
  limit?: number                     // Max results, 1-100 (default: 25)
}
```

### Parameter Details

- **recording** (optional, but required if no recording_mbid)
  - Recording name for search (e.g., "Paranoid Android")
  - Searches across all recordings in MusicBrainz

- **recording_mbid** (optional, but required if no recording)
  - Direct lookup by MBID (e.g., "6bf6f137-f7e5-4e40-880f-db35b3f9c272")
  - Faster and more accurate than name search

- **artist** (optional)
  - Filter results by artist name
  - Highly recommended to narrow down search results

- **find_appearances** (optional)
  - `false` (default): Return recording info only
  - `true`: Show all releases containing this recording

- **limit** (optional)
  - Range: 1-100
  - Default: 25
  - Applies to search results

**Note**: Either `recording` or `recording_mbid` must be provided.

---

## Examples

### Example 1: Search Recording by Name

Basic recording search with artist filter.

**Request**:
```json
{
  "name": "mb_recording_search",
  "arguments": {
    "recording": "Paranoid Android",
    "artist": "Radiohead"
  }
}
```

**Response** (truncated):
```
Recording Search Results
========================

Found 1 recording(s)

1. Paranoid Android
   MBID: 6bf6f137-f7e5-4e40-880f-db35b3f9c272
   Artist: Radiohead
   Duration: 6:23
   First Release: 1997
```

---

### Example 2: Find Where Recording Appears

Discover all releases containing a specific track.

**Request**:
```json
{
  "name": "mb_recording_search",
  "arguments": {
    "recording_mbid": "6bf6f137-f7e5-4e40-880f-db35b3f9c272",
    "find_appearances": true,
    "limit": 10
  }
}
```

**Response** (truncated):
```
Recording: Paranoid Android
MBID: 6bf6f137-f7e5-4e40-880f-db35b3f9c272
Artist: Radiohead
Duration: 6:23

This recording appears on 47 releases:

1. OK Computer (1997, Album)
   Release MBID: 52709206-8816-3c12-9ff6-f957f2f1eecf
   Track #2

2. OK Computer OKNOTOK 1997 2017 (2017, Album)
   Release MBID: ab96639e-b9e9-481c-abd6-8abd92e85661
   Track #2, Disc 1

3. The Best Of (2008, Compilation)
   Release MBID: c5a3e27c-0a87-4629-8f60-3f01f8c3f4ef
   Track #8

[... 44 more releases ...]
```

---

### Example 3: Search by MBID

Direct lookup when you already have the recording MBID.

**Request**:
```json
{
  "name": "mb_recording_search",
  "arguments": {
    "recording_mbid": "6bf6f137-f7e5-4e40-880f-db35b3f9c272"
  }
}
```

**Behavior**: Returns recording details instantly without searching.

---

## Use Cases

### 1. Identify Tracks Across Different Releases
Find the same recording in albums, compilations, and singles:
```json
{
  "recording": "Bohemian Rhapsody",
  "artist": "Queen",
  "find_appearances": true,
  "limit": 50
}
```

### 2. Find Live Versions, Covers, or Remixes
Search for variations of a track:
```json
{
  "recording": "One (live)",
  "artist": "U2"
}
```

### 3. Discover Compilation Albums
Track which compilations contain a specific song:
```json
{
  "recording_mbid": "...",
  "find_appearances": true
}
```

### 4. Track Down Rare or Region-Specific Releases
Find where a recording was released:
```json
{
  "recording": "B-side track name",
  "artist": "Artist Name",
  "find_appearances": true
}
```

### 5. Verify Recording Information
Confirm track details before tagging:
```json
{
  "recording": "Track Name",
  "artist": "Artist Name"
}
```

---

## Response Fields

### Recording Information

- **Title**: Recording name
- **MBID**: Unique MusicBrainz recording identifier
- **Artist**: Primary artist name(s)
- **Duration**: Track length (MM:SS)
- **First Release**: Year of first release
- **ISRC**: International Standard Recording Code (if available)

### Appearance Information (when find_appearances=true)

For each release containing the recording:
- **Release Title**: Album/single name
- **Release MBID**: Unique release identifier
- **Year**: Release year
- **Type**: Album, Single, Compilation, etc.
- **Track Position**: Track number and disc (if multi-disc)

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
1. mb_recording_search (recording: "name", artist: "name") → Get recording MBID
2. mb_recording_search (recording_mbid: ..., find_appearances: true) → Get releases
3. mb_release_search (release_mbid: ...) → Get full release details
```

### Pattern 2: Compilation Discovery
```
1. mb_recording_search (recording_mbid: "...", find_appearances: true, limit: 100)
2. Filter results by type: "Compilation"
3. Get compilation release details
```

### Pattern 3: Live/Alternative Version Discovery
```
1. Search base recording: mb_recording_search (recording: "Song Name", artist: "Artist")
2. Search variations: mb_recording_search (recording: "Song Name (live)", artist: "Artist")
3. Compare durations and appearances
```

---

## Tips

### Narrow Down Results
Always include the artist parameter to avoid ambiguous results:
```json
{
  "recording": "One",  // Too many results!
  "artist": "U2"       // Much better
}
```

### Find Rare B-Sides
Use `find_appearances` to locate singles and EPs:
```json
{
  "recording": "B-side name",
  "artist": "Artist",
  "find_appearances": true
}
```

### Verify Audio File Identity
Compare durations to ensure correct match:
```json
{
  "recording": "Track Name",
  "artist": "Artist"
}
```
Check that the returned duration matches your file.

---

## See Also

- [Common Concepts](common-concepts.md) - MBIDs, recordings vs tracks
- [Rate Limiting](rate-limiting.md) - API usage guidelines
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
