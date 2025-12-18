# mb_release_search

Search for releases (albums, singles, etc.) and optionally retrieve tracklists or alternative versions.

---

## Overview

The `mb_release_search` tool allows you to:
- Search for releases by name and artist
- Look up releases by MusicBrainz ID (MBID)
- Get complete tracklists with recording MBIDs
- Find all versions of a release (remasters, editions, countries)
- Access detailed release information (label, barcode, format)

---

## Parameters

```typescript
{
  release?: string,                    // Release name to search
  release_mbid?: string,               // Release MBID for direct lookup
  artist?: string,                     // Filter by artist name
  include_tracklist?: boolean,         // Include track listing (default: false)
  find_release_group_versions?: boolean, // Find other versions (default: false)
  limit?: number                       // Max results, 1-100 (default: 25)
}
```

### Parameter Details

- **release** (optional, but required if no release_mbid)
  - Release name for search (e.g., "OK Computer")
  - Searches across all releases in MusicBrainz

- **release_mbid** (optional, but required if no release)
  - Direct lookup by MBID (e.g., "52709206-8816-3c12-9ff6-f957f2f1eecf")
  - Faster and more accurate than name search

- **artist** (optional)
  - Filter results by artist name
  - Highly recommended to narrow down search results

- **include_tracklist** (optional)
  - `false` (default): Return release info only
  - `true`: Include complete track listing with durations and MBIDs

- **find_release_group_versions** (optional)
  - `false` (default): Show matching releases only
  - `true`: Find all versions (different countries, formats, reissues)

- **limit** (optional)
  - Range: 1-100
  - Default: 25
  - Applies to search results

**Note**: Either `release` or `release_mbid` must be provided.

---

## Examples

### Example 1: Search Release by Name

Basic release search with artist filter.

**Request**:
```json
{
  "name": "mb_release_search",
  "arguments": {
    "release": "OK Computer",
    "artist": "Radiohead"
  }
}
```

**Response** (truncated):
```
Release Search Results
======================

Found 25 release(s) (showing first 25)

1. OK Computer
   MBID: 52709206-8816-3c12-9ff6-f957f2f1eecf
   Artist: Radiohead
   Date: 1997-05-21
   Country: GB
   Format: CD
   Status: Official
   Packaging: Jewel Case
```

---

### Example 2: Get Release with Tracklist

Retrieve complete track information including durations and MBIDs.

**Request**:
```json
{
  "name": "mb_release_search",
  "arguments": {
    "release_mbid": "52709206-8816-3c12-9ff6-f957f2f1eecf",
    "include_tracklist": true
  }
}
```

**Response** (truncated):
```
Release: OK Computer
MBID: 52709206-8816-3c12-9ff6-f957f2f1eecf
Artist: Radiohead
Date: 1997-05-21
Country: GB
Label: Parlophone

Tracklist (12 tracks, 53:21):

 1. Airbag                      4:44
    Recording MBID: d4f52c25-e80e-4839-9484-8ce5a1c54d89

 2. Paranoid Android            6:23
    Recording MBID: 6bf6f137-f7e5-4e40-880f-db35b3f9c272

 3. Subterranean Homesick Alien 4:27
    Recording MBID: e2c3c210-70ee-4e12-8723-5c6a7f22c53e

[... tracks 4-12 ...]
```

---

### Example 3: Find Release Group Versions

Discover all editions of a release (remasters, different countries, formats).

**Request**:
```json
{
  "name": "mb_release_search",
  "arguments": {
    "release": "Discovery",
    "artist": "Daft Punk",
    "find_release_group_versions": true,
    "limit": 5
  }
}
```

**Response** shows:
- Original 2001 CD release (France)
- 2001 vinyl release (US)
- 2009 remaster (various countries)
- Digital releases
- Special editions

---

## Use Cases

### 1. Find Official Release Information
Get accurate release details for metadata tagging:
```json
{
  "release": "The Dark Side of the Moon",
  "artist": "Pink Floyd"
}
```

### 2. Get Complete Tracklists
Extract all tracks with MBIDs for a release:
```json
{
  "release_mbid": "...",
  "include_tracklist": true
}
```

### 3. Compare Different Editions
Find all versions to choose the right one:
```json
{
  "release": "Abbey Road",
  "artist": "The Beatles",
  "find_release_group_versions": true,
  "limit": 50
}
```

### 4. Verify Release Dates and Countries
Confirm which edition you have:
```json
{
  "release": "Random Access Memories",
  "artist": "Daft Punk",
  "find_release_group_versions": true
}
```

### 5. Extract Track MBIDs for Processing
Get recording identifiers for further operations:
```json
{
  "release_mbid": "...",
  "include_tracklist": true
}
```

---

## Response Fields

### Release Information

- **Title**: Release name
- **MBID**: Unique MusicBrainz release identifier
- **Artist**: Primary artist name(s)
- **Date**: Release date (YYYY-MM-DD or YYYY)
- **Country**: ISO country code
- **Label**: Record label name
- **Barcode**: Barcode/UPC if available
- **Format**: CD, Vinyl, Digital, Cassette, etc.
- **Status**: Official, Promotion, Bootleg, Pseudo-Release
- **Packaging**: Jewel Case, Digipak, Box, etc.

### Track Information (when include_tracklist=true)

- **Position**: Track number
- **Title**: Track name
- **Duration**: Track length (MM:SS)
- **Recording MBID**: Unique recording identifier
- **Artist**: Track artist (if different from release artist)

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
1. mb_release_search (release: "name", artist: "name") → Get MBID
2. mb_release_search (release_mbid: ..., include_tracklist: true) → Get full data
3. Use recording MBIDs for individual track processing
```

### Pattern 2: Find Best Release Version
```
1. mb_release_search (release: "name", artist: "name", find_release_group_versions: true)
2. Compare dates, countries, formats
3. Select preferred version's MBID
4. mb_release_search (release_mbid: ..., include_tracklist: true) → Get final data
```

### Pattern 3: Bulk Tracklist Export
```
1. mb_artist_search (artist: "name", include_releases: true) → Get all release MBIDs
2. For each release MBID:
   mb_release_search (release_mbid: ..., include_tracklist: true)
```

---

## See Also

- [Common Concepts](common-concepts.md) - MBIDs, release types, status codes
- [Rate Limiting](rate-limiting.md) - API usage guidelines
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
