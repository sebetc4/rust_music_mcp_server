# mb_advanced_search

Perform advanced searches using Lucene query syntax across all MusicBrainz entity types.

---

## Overview

The `mb_advanced_search` tool allows you to:
- Perform complex, multi-criteria searches
- Use Lucene query syntax for precise filtering
- Search across multiple entity types (artists, releases, recordings, etc.)
- Filter by specific fields not available in simple search
- Combine multiple conditions with boolean logic

---

## Parameters

```typescript
{
  entity: "artist" | "release" | "recording" | "release-group" | "work" | "label",
  query: string,      // Lucene query syntax
  limit?: number      // Max results, 1-100 (default: 25)
}
```

### Parameter Details

- **entity** (required)
  - Type of MusicBrainz entity to search
  - Options: `artist`, `release`, `recording`, `release-group`, `work`, `label`

- **query** (required)
  - Lucene-style query string
  - Supports field-specific searches, boolean operators, ranges, wildcards
  - See [Lucene Query Syntax](#lucene-query-syntax) below

- **limit** (optional)
  - Range: 1-100
  - Default: 25
  - Number of results to return

---

## Lucene Query Syntax

### Basic Operators

| Syntax | Example | Description |
|--------|---------|-------------|
| `field:value` | `country:US` | Field-specific search |
| `AND` | `artist:Radiohead AND country:GB` | Both conditions must match |
| `OR` | `type:Album OR type:EP` | Either condition matches |
| `NOT` | `artist:Beatles NOT status:bootleg` | Exclude matching items |
| `"phrase"` | `"dark side of the moon"` | Exact phrase match |
| `field:[min TO max]` | `date:[2020 TO 2023]` | Range query (inclusive) |
| `*` | `radio*` | Wildcard (matches any characters) |
| `?` | `wom?n` | Single character wildcard |
| `()` | `(type:Album OR type:EP) AND country:US` | Grouping |

### Operator Precedence

1. `NOT` (highest)
2. `AND`
3. `OR` (lowest)

Use parentheses `()` to control precedence.

---

## Available Fields by Entity

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

### Example 1: Find US-Released Albums from 2020

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "release",
    "query": "country:US AND date:[2020 TO 2020] AND status:Official",
    "limit": 10
  }
}
```

**Use Case**: Discover official US releases from a specific year.

---

### Example 2: Find Jazz Artists from France

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "artist",
    "query": "country:FR AND tag:jazz",
    "limit": 20
  }
}
```

**Use Case**: Genre and location-based artist discovery.

---

### Example 3: Find Recordings Between 3-4 Minutes

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "recording",
    "query": "artist:Radiohead AND dur:[180000 TO 240000]",
    "limit": 25
  }
}
```

**Note**: Duration is in milliseconds (180000ms = 3min, 240000ms = 4min).

---

### Example 4: Find Vinyl Releases

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "release",
    "query": "artist:\"Pink Floyd\" AND format:Vinyl AND status:Official",
    "limit": 15
  }
}
```

**Use Case**: Find specific format releases.

---

### Example 5: Find Recent Electronic Albums

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "release-group",
    "query": "tag:electronic AND type:Album AND date:[2020 TO 2024]",
    "limit": 30
  }
}
```

---

### Example 6: Find Active Female Artists

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "artist",
    "query": "type:Person AND gender:female AND end:*",
    "limit": 50
  }
}
```

**Note**: `end:*` finds artists without an end date (still active).

---

### Example 7: Find Singles from 2023

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "release",
    "query": "type:Single AND date:[2023-01-01 TO 2023-12-31] AND status:Official",
    "limit": 100
  }
}
```

---

### Example 8: Find Albums with Specific Track Count

**Request**:
```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "release",
    "query": "artist:\"The Beatles\" AND tracks:12 AND format:CD",
    "limit": 10
  }
}
```

---

## Use Cases

### 1. Complex Multi-Criteria Searches
Combine multiple conditions for precise results:
```json
{
  "entity": "release",
  "query": "artist:\"Miles Davis\" AND type:Album AND date:[1950 TO 1960] AND status:Official"
}
```

### 2. Filter by Specific Fields
Use fields not available in simple search:
```json
{
  "entity": "release",
  "query": "format:Vinyl AND country:JP"
}
```

### 3. Find Releases Meeting Exact Requirements
Perfect for collectors:
```json
{
  "entity": "release",
  "query": "artist:\"David Bowie\" AND format:Vinyl AND country:GB AND date:[1970 TO 1980]"
}
```

### 4. Discover Music by Genre/Style
Explore by tags:
```json
{
  "entity": "artist",
  "query": "tag:\"trip hop\" AND country:GB"
}
```

### 5. Research Specific Time Periods
Historical music research:
```json
{
  "entity": "recording",
  "query": "tag:blues AND date:[1920 TO 1940]"
}
```

### 6. Find Releases by Label
Label-specific searches:
```json
{
  "entity": "release",
  "query": "label:\"Blue Note\" AND type:Album AND date:[1955 TO 1965]"
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
