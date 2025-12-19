# mb_work_search

Search for **works** (musical compositions) in the MusicBrainz database.

---

## Overview

Works represent the underlying musical composition, independent of any specific recording or release. They are the abstract musical piece created by a composer.

**Use when**:
- Searching for classical compositions
- Finding all recordings of a specific song
- Researching musical compositions
- Tracking songwriting credits

**Related tools**:
- [mb_recording_search](mb_recording_search.md) - Find specific performances/recordings of a work
- [mb_artist_search](mb_artist_search.md) - Find the composer/songwriter

---

## Parameters

```typescript
interface MbWorkSearchParams {
  query: string;          // Work title to search for
  limit?: number;         // Max results (default: 10, max: 100)
}
```

### Field Details

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | ✅ Yes | - | Work title (e.g., "Bohemian Rhapsody", "Symphony No. 9") |
| `limit` | number | No | 10 | Maximum number of results (1-100) |

---

## Response Format

### Structured Output

```typescript
interface WorkSearchResult {
  works: WorkInfo[];
  total_count: number;
  query: string;
}

interface WorkInfo {
  title: string;              // Work title
  mbid: string;               // MusicBrainz Work ID
  work_type: string | null;   // Type (e.g., "Song", "Symphony")
  disambiguation: string | null;  // Disambiguation text
  language: string | null;    // Language code
}
```

### Text Summary

```
Found {count} work(s) matching '{query}'
```

---

## Examples

### Example 1: Search for a Popular Song

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "mb_work_search",
      "arguments": {
        "query": "Bohemian Rhapsody"
      }
    }
  }'
```

**Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "Found 1 work(s) matching 'Bohemian Rhapsody'"
  }],
  "structured_content": {
    "works": [{
      "title": "Bohemian Rhapsody",
      "mbid": "7b3c7e....",
      "work_type": "Song",
      "disambiguation": null,
      "language": "eng"
    }],
    "total_count": 1,
    "query": "Bohemian Rhapsody"
  }
}
```

### Example 2: Search for Classical Work

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "mb_work_search",
      "arguments": {
        "query": "Symphony No. 9",
        "limit": 5
      }
    }
  }'
```

### Example 3: Find Multiple Versions

```bash
# Search for a work with many interpretations
{
  "name": "mb_work_search",
  "arguments": {
    "query": "Canon in D",
    "limit": 3
  }
}
```

---

## Use Cases

### 1. Classical Music Research

```typescript
// Find a Beethoven symphony
{
  "query": "Symphony No. 5 in C minor",
  "limit": 10
}
```

### 2. Song Composition Credits

```typescript
// Find the original work to see composers
{
  "query": "Yesterday",
  "limit": 5
}
```

### 3. Musical Theater

```typescript
// Find a musical theater piece
{
  "query": "Memory from Cats",
  "limit": 3
}
```

---

## Common Patterns

### Pattern 1: Work → Recordings

```typescript
// Step 1: Find the work
const workResult = await callTool("mb_work_search", {
  query: "Bohemian Rhapsody"
});

// Step 2: Use the MBID to find recordings
// (Would require additional API calls to MusicBrainz)
const workMbid = workResult.works[0].mbid;
```

### Pattern 2: Disambiguation

```typescript
// When multiple works have the same name
{
  "query": "Hallelujah",  // Many different "Hallelujah" works
  "limit": 10
}
// Check 'disambiguation' field to identify the right one
```

---

## Field Reference

### Work Types

Common work types in MusicBrainz:
- `Song` - Popular music song
- `Symphony` - Orchestral symphony
- `Concerto` - Instrumental concerto
- `Opera` - Full opera work
- `Aria` - Solo vocal piece from an opera
- `Musical` - Musical theater work
- `Ballet` - Ballet composition

### Language Codes

ISO 639-3 language codes:
- `eng` - English
- `fra` - French
- `deu` - German
- `ita` - Italian
- `spa` - Spanish
- `jpn` - Japanese

---

## Troubleshooting

### No Results Found

**Problem**: Search returns empty results

**Solutions**:
1. Check spelling of work title
2. Try broader search terms (e.g., "Symphony 9" instead of "Symphony No. 9 in D minor")
3. Remove catalog numbers (e.g., "BWV 1007")
4. Search for alternate titles

### Too Many Results

**Problem**: Getting too many unrelated works

**Solutions**:
1. Add more specific terms to query
2. Include composer name in search
3. Reduce `limit` parameter
4. Check `disambiguation` field

### Work vs Recording Confusion

**Problem**: Not sure whether to search for work or recording

**Rule of thumb**:
- **Work**: The composition itself (e.g., "Moonlight Sonata")
- **Recording**: A specific performance (e.g., "Moonlight Sonata by Glenn Gould, 1981")

Use `mb_work_search` for the composition, `mb_recording_search` for performances.

---

## Related Documentation

- [mb_recording_search.md](mb_recording_search.md) - Find recordings of works
- [mb_artist_search.md](mb_artist_search.md) - Find composers/songwriters
- [common-concepts.md](common-concepts.md#works) - Understanding works in MusicBrainz
- [rate-limiting.md](rate-limiting.md) - API rate limits

---

## Technical Notes

### Implementation

- **File**: `src/domains/tools/definitions/mb/work.rs`
- **API**: MusicBrainz `/work` search endpoint
- **Rate Limit**: 1 request/second (MusicBrainz)

### Performance

- Typical response time: 200-500ms
- Results are sorted by relevance
- Limit parameter affects only returned results, not search performance

---

## See Also

- [MusicBrainz Work Documentation](https://musicbrainz.org/doc/Work)
- [Work Entity in MusicBrainz](https://musicbrainz.org/doc/Work)
- [Common Concepts](common-concepts.md)
