# mb_artist_search

Search for artists in the MusicBrainz database and optionally retrieve their releases.

---

## Overview

The `mb_artist_search` tool allows you to:
- Search for artists by name
- Look up artists by MusicBrainz ID (MBID)
- Retrieve an artist's complete discography
- Get detailed artist information (type, country, active period)

---

## Parameters

```typescript
{
  artist: string,              // Artist name or MBID (required)
  include_releases?: boolean,  // Include artist's releases (default: false)
  limit?: number              // Max results, 1-100 (default: 25)
}
```

### Parameter Details

- **artist** (required)
  - Artist name for search (e.g., "Radiohead")
  - Or MBID for direct lookup (e.g., "a74b1b7f-71a5-4011-9441-d0b5e4122711")
  - Tool automatically detects MBID format and performs direct lookup

- **include_releases** (optional)
  - `false` (default): Return artist info only
  - `true`: Include complete discography with releases

- **limit** (optional)
  - Range: 1-100
  - Default: 25
  - Applies to both artist results and releases per artist

---

## Examples

### Example 1: Simple Artist Search

Search for an artist by name to get basic information.

**Request**:
```json
{
  "name": "mb_artist_search",
  "arguments": {
    "artist": "Radiohead"
  }
}
```

**Response**:
```
Artist Search Results for 'Radiohead'
=====================================

Found 1 artist(s)

1. Radiohead
   MBID: a74b1b7f-71a5-4011-9441-d0b5e4122711
   Type: Group
   Country: GB
   Active: 1991 - present
   Disambiguation: UK rock band
```

---

### Example 2: Artist Search with Releases

Get an artist's complete discography.

**Request**:
```json
{
  "name": "mb_artist_search",
  "arguments": {
    "artist": "Daft Punk",
    "include_releases": true,
    "limit": 10
  }
}
```

**Response** (truncated):
```
Artist Search Results for 'Daft Punk'
======================================

Found 1 artist(s)

1. Daft Punk
   MBID: 056e4f3e-d505-4dad-8ec1-d04f521cbb56
   Type: Group
   Country: FR
   Active: 1993 - 2021
   Disambiguation: French electronic music duo

   Releases (showing first 10):

   - Random Access Memories (2013) [Album]
     MBID: f3dc77fa-c7c8-4e03-b4b7-cb2b36e2eaf0

   - Discovery (2001) [Album]
     MBID: b81bcdb6-4223-43e9-a6a3-90537f8c0eb5

   - Homework (1997) [Album]
     MBID: dc16fd5a-0662-4bcf-8dfb-7f5ce8a1c7b2
```

---

### Example 3: Direct Lookup by MBID

When you already have the MBID, get instant results without searching.

**Request**:
```json
{
  "name": "mb_artist_search",
  "arguments": {
    "artist": "a74b1b7f-71a5-4011-9441-d0b5e4122711",
    "include_releases": true
  }
}
```

**Behavior**: The tool detects the MBID format and performs a direct lookup instead of a search, which is faster and more accurate.

---

## Use Cases

### 1. Find Artist MBID
Get the unique identifier for further queries or tagging:
```json
{
  "artist": "Massive Attack"
}
```

### 2. Discover Artist Discography
Explore an artist's complete catalog:
```json
{
  "artist": "Aphex Twin",
  "include_releases": true,
  "limit": 50
}
```

### 3. Verify Artist Information
Confirm details like country, type, or active period:
```json
{
  "artist": "The Beatles"
}
```

### 4. Explore Similar-Named Artists
Find artists with similar names by increasing the limit:
```json
{
  "artist": "Black Star",
  "limit": 50
}
```

---

## Response Fields

### Artist Information

- **Name**: Artist or band name
- **MBID**: Unique MusicBrainz identifier
- **Type**: Person, Group, Orchestra, Choir, Character, Other
- **Country**: ISO country code (e.g., GB, US, FR)
- **Active**: Begin year - end year (or "present")
- **Disambiguation**: Additional context to distinguish similar artists

### Release Information (when include_releases=true)

- **Title**: Release name
- **Year**: Release year
- **Type**: Album, Single, EP, Compilation, etc.
- **MBID**: Release identifier for further queries

---

## Related Tools

- [mb_release_search](mb_release_search.md) - Get detailed release information
- [mb_advanced_search](mb_advanced_search.md) - Complex artist queries with filters

---

## Common Patterns

### Pattern 1: Artist Identification Pipeline
```
1. mb_artist_search (artist: "name") → Get MBID
2. mb_artist_search (artist: MBID, include_releases: true) → Get discography
3. mb_release_search (release_mbid: ..., include_tracklist: true) → Get tracks
```

### Pattern 2: Discography Export
```
1. mb_artist_search (artist: "name", include_releases: true, limit: 100)
2. For each release → mb_release_search to get full details
```

---

## See Also

- [Common Concepts](common-concepts.md) - MBIDs, limits, entity types
- [Rate Limiting](rate-limiting.md) - API usage guidelines
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
