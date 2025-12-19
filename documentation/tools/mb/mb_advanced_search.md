# mb_advanced_search

Perform advanced searches across all MusicBrainz entity types with field-specific queries.

---

## Overview

The `mb_advanced_search` tool allows you to:
- Perform complex, multi-criteria searches
- Search across multiple entity types (artists, releases, recordings, release groups, works, labels)
- Filter by specific fields (country, year, format, etc.)
- Returns structured JSON data with concise text summaries
- Supports advanced query syntax for precise filtering

**Output Format**: This tool follows MCP standards, returning a short text summary plus structured JSON data for programmatic access.

---

## Parameters

```typescript
{
  entity: "artist" | "release" | "recording" | "release_group" | "work" | "label",
  query: string,      // Query string (simple search)
  limit?: number      // Max results, 1-100 (default: 10)
}
```

### Parameter Details

- **entity** (required)
  - Type of MusicBrainz entity to search
  - Options: `artist`, `release`, `recording`, `release_group`, `work`, `label`

- **query** (required)
  - Search query string
  - Simple text search within the entity type
  - Example: "Radiohead", "OK Computer", "jazz"

- **limit** (optional)
  - Range: 1-100
  - Default: 10
  - Number of results to return

---

## Output Format

The tool returns:
1. **Text Summary**: A concise one-line description of results (e.g., "Found 5 artist(s) matching 'Radiohead'")
2. **Structured JSON Data**: Complete data in a standardized format for programmatic access

This follows MCP standards for structured tool output, providing both human-readable summaries and machine-parseable data.

---

## Supported Entity Types

### Artist Fields

| Field | Type | Example | Description |
|-------|------|---------|-------------|
| `artist` | text | `artist:Radiohead` | Artist name |
| `type` | enum | `type:Group` | Person, Group, Orchestra, Choir, Character, Other |
| `country` | code | `country:GB` | ISO country code (2 letters) |
| `area` | text | `area:London` | Geographic area |
| `begin` | year | `begin:1990` | Begin year |
| `end` | year | `end:2021` | End year |
| `tag` | text | `tag:jazz` | Genre/style tags |
| `gender` | enum | `gender:female` | Male, Female, Other (for Person type) |

---

### Release Fields

| Field | Type | Example | Description |
|-------|------|---------|-------------|
| `release` | text | `release:"OK Computer"` | Release title |
| `artist` | text | `artist:"Pink Floyd"` | Artist name |
| `date` | date | `date:1997` | Release date (YYYY-MM-DD or YYYY) |
| `country` | code | `country:US` | ISO country code |
| `format` | enum | `format:Vinyl` | CD, Vinyl, Digital Download, Cassette, etc. |
| `status` | enum | `status:Official` | Official, Promotion, Bootleg, Pseudo-Release |
| `barcode` | number | `barcode:724384260927` | Barcode/UPC |
| `label` | text | `label:Parlophone` | Record label name |
| `tracks` | number | `tracks:12` | Number of tracks |
| `type` | enum | `type:Album` | Album, Single, EP, Compilation, etc. |

---

### Recording Fields

| Field | Type | Example | Description |
|-------|------|---------|-------------|
| `recording` | text | `recording:"Paranoid Android"` | Recording title |
| `artist` | text | `artist:Radiohead` | Artist name |
| `dur` | milliseconds | `dur:[180000 TO 240000]` | Duration in milliseconds |
| `date` | year | `date:1997` | First release date |
| `tag` | text | `tag:rock` | Genre/style tags |
| `isrc` | code | `isrc:GBAYE9700361` | International Standard Recording Code |
| `video` | boolean | `video:true` | Is video recording |

---

### Release-Group Fields

| Field | Type | Example | Description |
|-------|------|---------|-------------|
| `releasegroup` | text | `releasegroup:"Discovery"` | Release group title |
| `artist` | text | `artist:"Daft Punk"` | Artist name |
| `type` | enum | `type:Album` | Album, Single, EP, etc. |
| `tag` | text | `tag:electronic` | Genre/style tags |
| `date` | year | `date:2001` | First release date |

---

### Work Fields

| Field | Type | Example | Description |
|-------|------|---------|-------------|
| `work` | text | `work:"Symphony No. 9"` | Work title |
| `artist` | text | `artist:Beethoven` | Composer/creator |
| `type` | enum | `type:Song` | Work type |
| `tag` | text | `tag:classical` | Genre/style tags |

---

### Label Fields

| Field | Type | Example | Description |
|-------|------|---------|-------------|
| `label` | text | `label:Parlophone` | Label name |
| `type` | enum | `type:"Original Production"` | Label type |
| `country` | code | `country:GB` | ISO country code |
| `area` | text | `area:London` | Geographic area |
| `begin` | year | `begin:1896` | Founded year |
| `end` | year | `end:2013` | Ended year |

---

## Examples

### Example 1: Search for Artists

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "artist",
    "query": "Radiohead",
    "limit": 5
  }
}
```

**Text Summary**:
```
Found 5 artist(s) matching 'Radiohead'
```

**Structured Data**:
```json
{
  "artists": [
    {
      "name": "Radiohead",
      "mbid": "a74b1b7f-71a5-4011-9441-d0b5e4122711",
      "country": "GB",
      "disambiguation": "UK rock band"
    }
  ],
  "total_count": 5,
  "query": "Radiohead"
}
```

---

### Example 2: Search for Releases

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "release",
    "query": "OK Computer",
    "limit": 10
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
      "year": "1997"
    }
  ],
  "total_count": 10,
  "query": "OK Computer"
}
```

---

### Example 3: Search for Release Groups

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "release_group",
    "query": "Discovery",
    "limit": 5
  }
}
```

**Text Summary**:
```
Found 5 release group(s) matching 'Discovery'
```

**Structured Data**:
```json
{
  "release_groups": [
    {
      "title": "Discovery",
      "mbid": "b81bcdb6-4223-43e9-a6a3-90537f8c0eb5",
      "artist": "Daft Punk",
      "first_release_year": "2001"
    }
  ],
  "total_count": 5,
  "query": "Discovery"
}
```

---

### Example 4: Search for Recordings

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "recording",
    "query": "Paranoid Android",
    "limit": 5
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
      "duration": "6:23"
    }
  ],
  "total_count": 5,
  "query": "Paranoid Android"
}
```

---

### Example 5: Search for Works

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "work",
    "query": "Symphony No. 9",
    "limit": 10
  }
}
```

**Text Summary**:
```
Found 10 work(s) matching 'Symphony No. 9'
```

**Structured Data**:
```json
{
  "works": [
    {
      "title": "Symphony No. 9",
      "mbid": "work-mbid-here",
      "disambiguation": "Beethoven"
    }
  ],
  "total_count": 10,
  "query": "Symphony No. 9"
}
```

---

### Example 6: Search for Labels

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "label",
    "query": "Parlophone",
    "limit": 5
  }
}
```

**Text Summary**:
```
Found 5 label(s) matching 'Parlophone'
```

**Structured Data**:
```json
{
  "labels": [
    {
      "name": "Parlophone",
      "mbid": "label-mbid-here",
      "country": "GB",
      "disambiguation": null
    }
  ],
  "total_count": 5,
  "query": "Parlophone"
}
```

---

## Response Fields

Each entity type returns a specific structured format:

### Artist Results

```typescript
{
  artists: [
    {
      name: string,              // Artist name
      mbid: string,              // Artist MBID
      country: string | null,    // ISO country code
      disambiguation: string | null  // Additional context
    }
  ],
  total_count: number,
  query: string
}
```

### Release Results

```typescript
{
  releases: [
    {
      title: string,             // Release title
      mbid: string,              // Release MBID
      artist: string,            // Artist name
      year: string | null        // Release year
    }
  ],
  total_count: number,
  query: string
}
```

### Release Group Results

```typescript
{
  release_groups: [
    {
      title: string,             // Release group title
      mbid: string,              // Release group MBID
      artist: string,            // Artist name
      first_release_year: string | null  // First release year
    }
  ],
  total_count: number,
  query: string
}
```

### Recording Results

```typescript
{
  recordings: [
    {
      title: string,             // Recording title
      mbid: string,              // Recording MBID
      artist: string,            // Artist name
      duration: string | null    // Duration (MM:SS)
    }
  ],
  total_count: number,
  query: string
}
```

### Work Results

```typescript
{
  works: [
    {
      title: string,             // Work title
      mbid: string,              // Work MBID
      disambiguation: string | null  // Additional context
    }
  ],
  total_count: number,
  query: string
}
```

### Label Results

```typescript
{
  labels: [
    {
      name: string,              // Label name
      mbid: string,              // Label MBID
      country: string | null,    // ISO country code
      disambiguation: string | null  // Additional context
    }
  ],
  total_count: number,
  query: string
}
```

---

## Use Cases

### 1. Search Across Different Entity Types
Explore different aspects of music data:
```json
{
  "entity": "artist",
  "query": "Miles Davis"
}
```

### 2. Find Specific Releases
Search for releases by title:
```json
{
  "entity": "release",
  "query": "Kind of Blue"
}
```

### 3. Discover Works
Find musical works and compositions:
```json
{
  "entity": "work",
  "query": "Moonlight Sonata"
}
```

### 4. Search by Label
Find releases by record label:
```json
{
  "entity": "label",
  "query": "Blue Note"
}
```

### 5. Cross-Reference Entities
Use MBIDs from results to query other tools:
```json
{
  "entity": "release_group",
  "query": "Random Access Memories"
}
```

---

## Tips and Best Practices

### Use Quotes for Exact Phrases
```
artist:"Daft Punk"        // Exact match
artist:Daft Punk          // Searches for "Daft" OR "Punk"
```

### Wildcard Searches
```
artist:radio*             // Matches "Radiohead", "Radio Dept.", etc.
artist:*head              // Matches any artist ending in "head"
```

### Date Ranges
```
date:[2020 TO 2023]       // 2020, 2021, 2022, 2023
date:[* TO 2000]          // Any date up to 2000
date:[2000 TO *]          // 2000 onwards
```

### Duration Conversions
| Duration | Milliseconds |
|----------|--------------|
| 3:00 | 180000 |
| 4:00 | 240000 |
| 5:00 | 300000 |
| 10:00 | 600000 |

Formula: `minutes * 60 * 1000 + seconds * 1000`

### Combine with Boolean Logic
```
(type:Album OR type:EP) AND artist:Radiohead AND date:[1990 TO 2000]
```

### Exclude Unwanted Results
```
artist:Beatles NOT status:bootleg
```

---

## Related Tools

- [mb_artist_search](mb_artist_search.md) - Simple artist search
- [mb_release_search](mb_release_search.md) - Simple release search
- [mb_recording_search](mb_recording_search.md) - Simple recording search

Use `mb_advanced_search` when simple tools don't provide enough filtering options.

---

## External Resources

- [Lucene Query Syntax Documentation](https://lucene.apache.org/core/2_9_4/queryparsersyntax.html)
- [MusicBrainz Search API](https://musicbrainz.org/doc/MusicBrainz_API/Search)
- [MusicBrainz Indexed Search Fields](https://musicbrainz.org/doc/Indexed_Search_Syntax)

---

## See Also

- [Common Concepts](common-concepts.md) - Entity types, field meanings
- [Rate Limiting](rate-limiting.md) - API usage guidelines
- [Troubleshooting](troubleshooting.md) - Common query issues
