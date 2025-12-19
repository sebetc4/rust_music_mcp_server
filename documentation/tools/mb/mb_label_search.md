# mb_label_search

Search for **labels** (record labels/publishers) in the MusicBrainz database.

---

## Overview

Labels represent companies or organizations that publish music releases. They can be major record labels, independent labels, or even self-publishing entities.

**Use when**:
- Finding information about a record label
- Researching label discographies
- Identifying label ownership
- Tracking release history by label

**Related tools**:
- [mb_release_search](mb_release_search.md) - Find releases by a label
- [mb_artist_search](mb_artist_search.md) - Find artists on a label

---

## Parameters

```typescript
interface MbLabelSearchParams {
  query: string;          // Label name to search for
  limit?: number;         // Max results (default: 10, max: 100)
}
```

### Field Details

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | ✅ Yes | - | Label name (e.g., "Sony Music", "XL Recordings") |
| `limit` | number | No | 10 | Maximum number of results (1-100) |

---

## Response Format

### Structured Output

```typescript
interface LabelSearchResult {
  labels: LabelInfo[];
  total_count: number;
  query: string;
}

interface LabelInfo {
  name: string;                // Label name
  mbid: string;                // MusicBrainz Label ID
  label_type: string | null;   // Label type (e.g., "Original Production")
  country: string | null;      // Country code (ISO 3166-1)
  disambiguation: string | null;  // Disambiguation text
  label_code: number | null;   // LC code (Label Code)
}
```

### Text Summary

```
Found {count} label(s) matching '{query}'
```

---

## Examples

### Example 1: Search for Major Label

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "mb_label_search",
      "arguments": {
        "query": "Sony Music"
      }
    }
  }'
```

**Response**:
```json
{
  "content": [{
    "type": "text",
    "text": "Found 10 label(s) matching 'Sony Music'"
  }],
  "structured_content": {
    "labels": [{
      "name": "Sony Music Entertainment",
      "mbid": "9e6b4d7a-...",
      "label_type": "OriginalProduction",
      "country": "US",
      "disambiguation": "multinational music company",
      "label_code": 5199
    }],
    "total_count": 10,
    "query": "Sony Music"
  }
}
```

### Example 2: Search for Independent Label

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "mb_label_search",
      "arguments": {
        "query": "XL Recordings",
        "limit": 5
      }
    }
  }'
```

### Example 3: Search by Label Code

```bash
# Search using label code
{
  "name": "mb_label_search",
  "arguments": {
    "query": "LC 0199",  // Blue Note Records label code
    "limit": 3
  }
}
```

---

## Use Cases

### 1. Label Research

```typescript
// Find a label to explore their catalog
{
  "query": "Motown Records",
  "limit": 5
}
```

### 2: Identify Label from Release

```typescript
// Found "4AD" on a release, get more info
{
  "query": "4AD",
  "limit": 10
}
```

### 3. Regional Label Discovery

```typescript
// Find labels from a specific region
{
  "query": "Warp Records",  // UK electronic label
  "limit": 5
}
```

---

## Common Patterns

### Pattern 1: Label → Releases

```typescript
// Step 1: Find the label
const labelResult = await callTool("mb_label_search", {
  query: "Blue Note"
});

// Step 2: Use label MBID to find releases
const labelMbid = labelResult.labels[0].mbid;
// (Would require additional MusicBrainz API calls)
```

### Pattern 2: Disambiguation

```typescript
// When multiple labels share the same name
{
  "query": "Columbia",
  "limit": 10
}
// Check 'country' and 'disambiguation' fields
// - "Columbia Records" (US)
// - "Columbia Graphophone Company" (UK, historical)
```

### Pattern 3: Subsidiary Labels

```typescript
// Find parent company labels and subsidiaries
{
  "query": "Universal Music",
  "limit": 20
}
```

---

## Field Reference

### Label Types

Common label types in MusicBrainz:
- `OriginalProduction` - Original production label
- `Bootleg Production` - Unofficial/bootleg label
- `Reissue Production` - Reissue label
- `Distribution` - Distribution company
- `Publisher` - Music publisher
- `Holding` - Holding company

### Country Codes

ISO 3166-1 alpha-2 codes:
- `US` - United States
- `GB` - United Kingdom
- `DE` - Germany
- `FR` - France
- `JP` - Japan
- `CA` - Canada

### Label Code (LC)

International label code assigned by IFPI:
- Format: `LC xxxx` or just the number
- Example: `LC 0199` = Blue Note Records
- Used for rights management and royalties

---

## Troubleshooting

### No Results Found

**Problem**: Search returns empty results

**Solutions**:
1. Check label name spelling
2. Try abbreviated form (e.g., "EMI" instead of "EMI Records")
3. Remove country suffix (e.g., "Sony" instead of "Sony Music USA")
4. Search for parent company name

### Too Many Generic Results

**Problem**: Getting unrelated labels

**Solutions**:
1. Add more specific terms (e.g., "Verve Records" instead of just "Verve")
2. Include country or type in query
3. Use `disambiguation` field to identify correct label
4. Check `label_code` for verification

### Imprint vs Parent Label

**Problem**: Confused about label relationships

**Explanation**:
- **Parent label**: Main company (e.g., "Universal Music Group")
- **Imprint**: Sub-label (e.g., "Interscope Records")

Both are searchable as separate labels in MusicBrainz.

---

## Advanced Usage

### Filter by Country

```typescript
// Search and filter results by country
{
  "query": "Atlantic",
  "limit": 20
}
// Then filter structured_content.labels by country === "US"
```

### Verify Label Authenticity

```typescript
// Use label_code to verify official label
{
  "query": "Blue Note Records"
}
// Check: label_code === 199 confirms it's the official label
```

---

## Related Documentation

- [mb_release_search.md](mb_release_search.md) - Find releases by label
- [mb_artist_search.md](mb_artist_search.md) - Find artists on a label
- [common-concepts.md](common-concepts.md#labels) - Understanding labels
- [rate-limiting.md](rate-limiting.md) - API rate limits

---

## Technical Notes

### Implementation

- **File**: `src/domains/tools/definitions/mb/label.rs`
- **API**: MusicBrainz `/label` search endpoint
- **Rate Limit**: 1 request/second (MusicBrainz)

### Performance

- Typical response time: 200-500ms
- Results sorted by relevance
- Label codes are integers (converted from u32)

### Data Quality

- Some labels may lack `label_code`
- `disambiguation` helps identify the correct label
- Historical labels may have `label_type` = null

---

## See Also

- [MusicBrainz Label Documentation](https://musicbrainz.org/doc/Label)
- [Label Code (LC) System](https://www.ifpi.org/)
- [Common Concepts](common-concepts.md)
