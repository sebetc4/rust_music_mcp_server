# mb_artist_search

Search for artists in the MusicBrainz database and optionally retrieve their releases.

---

## Overview

The `mb_artist_search` tool allows you to:
- Search for artists by name
- Retrieve an artist's complete discography
- Get detailed artist information (country, area, disambiguation)
- Returns structured JSON data with concise text summaries

**Output Format**: This tool follows MCP standards, returning a short text summary plus structured JSON data for programmatic access.

---

## Parameters

```typescript
{
  search_type: "artist" | "artist_releases",  // Type of search (required)
  query: string,                               // Artist name or MBID (required)
  limit?: number                               // Max results, 1-100 (default: 10)
}
```

### Parameter Details

- **search_type** (required)
  - `"artist"`: Search for artists by name or fetch by MBID
  - `"artist_releases"`: Search for releases by a specific artist (by name or MBID)

- **query** (required)
  - Artist name for search (e.g., "Radiohead")
  - Or artist MBID for direct lookup (e.g., "a74b1b7f-71a5-4011-9441-d0b5e4122711")
  - Tool automatically detects MBID format and performs direct fetch instead of search

- **limit** (optional)
  - Range: 1-100
  - Default: 10
  - Applies to both artist results and releases per artist

---

## Output Format

The tool returns:
1. **Text Summary**: A concise one-line description of results (e.g., "Found 5 artist(s) matching 'Radiohead'")
2. **Structured JSON Data**: Complete data in a standardized format for programmatic access

This follows MCP standards for structured tool output, providing both human-readable summaries and machine-parseable data.

---

## Examples

### Example 1: Simple Artist Search

Search for an artist by name to get basic information.

**Request**:
```json
{
  "name": "mb_artist_search",
  "arguments": {
    "search_type": "artist",
    "query": "Radiohead"
  }
}
```

**Text Summary**:
```
Found 1 artist(s) matching 'Radiohead'
```

**Structured Data**:
```json
{
  "artists": [
    {
      "name": "Radiohead",
      "mbid": "a74b1b7f-71a5-4011-9441-d0b5e4122711",
      "country": "GB",
      "area": "Oxford",
      "disambiguation": "UK rock band"
    }
  ],
  "total_count": 1,
  "query": "Radiohead"
}
```

---

### Example 2: Artist Search with Releases

Get an artist's complete discography.

**Request**:
```json
{
  "name": "mb_artist_search",
  "arguments": {
    "search_type": "artist_releases",
    "query": "Daft Punk",
    "limit": 5
  }
}
```

**Text Summary**:
```
Found 5 release(s) by 'Daft Punk'
```

**Structured Data**:
```json
{
  "artist_name": "Daft Punk",
  "artist_mbid": "056e4f3e-d505-4dad-8ec1-d04f521cbb56",
  "releases": [
    {
      "title": "Random Access Memories",
      "mbid": "f3dc77fa-c7c8-4e03-b4b7-cb2b36e2eaf0",
      "year": "2013",
      "country": "FR"
    },
    {
      "title": "Discovery",
      "mbid": "b81bcdb6-4223-43e9-a6a3-90537f8c0eb5",
      "year": "2001",
      "country": "FR"
    },
    {
      "title": "Homework",
      "mbid": "dc16fd5a-0662-4bcf-8dfb-7f5ce8a1c7b2",
      "year": "1997",
      "country": "FR"
    }
  ],
  "total_count": 5
}
```

---

### Example 3: Direct Artist Lookup by MBID

When you already have an artist MBID, get instant artist information without searching.

**Request**:
```json
{
  "name": "mb_artist_search",
  "arguments": {
    "search_type": "artist",
    "query": "5b11f4ce-a62d-471e-81fc-a69a8278c7da"
  }
}
```

**Text Summary**:
```
Found artist: 'Nirvana'
```

**Structured Data**:
```json
{
  "artists": [
    {
      "name": "Nirvana",
      "mbid": "5b11f4ce-a62d-471e-81fc-a69a8278c7da",
      "country": "US",
      "area": "Aberdeen",
      "disambiguation": "90s US grunge band"
    }
  ],
  "total_count": 1,
  "query": "5b11f4ce-a62d-471e-81fc-a69a8278c7da"
}
```

**Behavior**: The tool detects the MBID format and performs a direct fetch instead of a search, which is faster and more accurate.

---

### Example 4: Direct Discography Lookup by MBID

When you already have an artist MBID, get their discography directly.

**Request**:
```json
{
  "name": "mb_artist_search",
  "arguments": {
    "search_type": "artist_releases",
    "query": "a74b1b7f-71a5-4011-9441-d0b5e4122711"
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
  "search_type": "artist",
  "query": "Massive Attack"
}
```

### 2. Discover Artist Discography
Explore an artist's complete catalog:
```json
{
  "search_type": "artist_releases",
  "query": "Aphex Twin",
  "limit": 50
}
```

### 3. Verify Artist Information
Confirm details like country or area:
```json
{
  "search_type": "artist",
  "query": "The Beatles"
}
```

### 4. Explore Similar-Named Artists
Find artists with similar names by increasing the limit:
```json
{
  "search_type": "artist",
  "query": "Black Star",
  "limit": 50
}
```

---

## Response Fields

### Artist Search (`search_type: "artist"`)

Returns `ArtistSearchResult` with:

```typescript
{
  artists: [
    {
      name: string,              // Artist or band name
      mbid: string,              // Unique MusicBrainz identifier
      country: string | null,    // ISO country code (e.g., "GB", "US", "FR")
      area: string | null,       // Geographic area (e.g., "Oxford", "London")
      disambiguation: string | null  // Additional context to distinguish similar artists
    }
  ],
  total_count: number,           // Number of artists returned
  query: string                  // Original search query
}
```

### Artist Releases Search (`search_type: "artist_releases"`)

Returns `ArtistReleasesResult` with:

```typescript
{
  artist_name: string,           // Artist name
  artist_mbid: string,           // Artist MBID
  releases: [
    {
      title: string,             // Release name
      mbid: string,              // Release identifier for further queries
      year: string | null,       // Release year (e.g., "2013")
      country: string | null     // ISO country code
    }
  ],
  total_count: number            // Number of releases returned
}
```

---

## Related Tools

- [mb_release_search](mb_release_search.md) - Get detailed release information
- [mb_advanced_search](mb_advanced_search.md) - Complex artist queries with filters

---

## Common Patterns

### Pattern 1: Artist Identification Pipeline
```
1. mb_artist_search (search_type: "artist", query: "name") → Get MBID
2. mb_artist_search (search_type: "artist_releases", query: MBID) → Get discography
3. mb_release_search (search_type: "release_recordings", query: release_mbid) → Get tracks
```

### Pattern 2: Discography Export
```
1. mb_artist_search (search_type: "artist_releases", query: "name", limit: 100)
2. For each release → mb_release_search to get full details
```

---

## See Also

- [Common Concepts](common-concepts.md) - MBIDs, limits, entity types
- [Rate Limiting](rate-limiting.md) - API usage guidelines
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
