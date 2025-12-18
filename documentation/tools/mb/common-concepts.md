# Common Concepts

Shared concepts and terminology used across all MusicBrainz tools.

---

## MusicBrainz Identifiers (MBIDs)

### Format

MBIDs are UUID-format identifiers consisting of 36 characters:

```
a74b1b7f-71a5-4011-9441-d0b5e4122711
```

**Structure**: 8-4-4-4-12 hexadecimal digits

### Properties

- **Unique**: Each entity has exactly one MBID
- **Permanent**: MBIDs never change, even if metadata is corrected
- **Universal**: Can be used across all MusicBrainz tools and external applications
- **Unambiguous**: No name confusion (e.g., multiple artists named "The Who")

### Usage

**Preferred over names for lookups**:
```json
// Less precise (searches by name)
{"artist": "Michael Jackson"}

// More precise (direct lookup by MBID)
{"artist": "f27ec8db-af05-4f36-916e-3d57f91ecf5e"}
```

### Validation

All tools automatically detect MBID format:
- If input matches UUID pattern → Direct lookup
- Otherwise → Search by name

### Entity Types

Each MBID type corresponds to a MusicBrainz entity:
- **Artist MBID**: Musicians, bands, composers
- **Release MBID**: Albums, singles, EPs (specific editions)
- **Recording MBID**: Individual track performances
- **Release Group MBID**: Abstract album (all editions combined)
- **Work MBID**: Musical composition
- **Label MBID**: Record labels

---

## Limit Parameter

### Overview

All search tools accept a `limit` parameter to control result count.

### Specifications

- **Type**: Integer
- **Range**: 1-100
- **Default**: 25
- **Behavior**: Values outside range are clamped to min/max

### Usage

```json
// Default (25 results)
{"artist": "Beatles"}

// Custom limit (50 results)
{"artist": "Beatles", "limit": 50}

// Maximum (100 results)
{"artist": "Beatles", "limit": 100}

// Minimum (1 result)
{"artist": "Beatles", "limit": 1}
```

### Recommendations

- **Quick lookups**: 1-10
- **Standard searches**: 25 (default)
- **Exploratory searches**: 50-100
- **Batch processing**: Start with 25, increase if needed

### Performance Considerations

Higher limits:
- ✅ More complete results
- ❌ Slower API responses
- ❌ More data to process
- ❌ Higher rate limit impact

---

## Release Status

Indicates the legitimacy and distribution method of a release.

| Status | Description | Examples |
|--------|-------------|----------|
| **Official** | Legitimate, authorized release | Standard albums, officially released singles |
| **Promotion** | Promotional release (not for sale) | Radio promos, press kits, advance copies |
| **Bootleg** | Unauthorized, unofficial release | Fan recordings, leaked albums, counterfeit pressings |
| **Pseudo-Release** | Non-physical or special | Streaming-only releases, digital singles |

### Usage in Queries

```json
// Only official releases
{"release": "OK Computer", "artist": "Radiohead"}

// Advanced search for official releases only
{
  "entity": "release",
  "query": "artist:Radiohead AND status:Official"
}
```

### Filtering Recommendations

- **For accurate tagging**: Use `status:Official`
- **For collectors**: Include `Promotion`
- **For completeness**: Include all statuses
- **Avoid**: `Bootleg` for standard library organization

---

## Release Types

Categorizes releases by format and purpose.

| Type | Description | Typical Track Count | Duration |
|------|-------------|---------------------|----------|
| **Album** | Standard full-length album | 8-15 tracks | 30-70 minutes |
| **Single** | Single track release | 1-4 tracks | 3-15 minutes |
| **EP** | Extended play | 3-6 tracks | 10-25 minutes |
| **Compilation** | Tracks from various sources | 10-30 tracks | 40-120 minutes |
| **Soundtrack** | Music from film/TV/game | 10-30 tracks | 30-120 minutes |
| **Live** | Recorded at live performance | Varies | Varies |
| **Remix** | Remixed versions of tracks | Varies | Varies |
| **Broadcast** | Radio/TV broadcast | Varies | Varies |
| **Other** | Doesn't fit standard categories | Varies | Varies |

### Sub-Categories

Some types have additional qualifications:
- **Album + Compilation**: Greatest hits, best of
- **Album + Live**: Live concert albums
- **Album + Soundtrack**: Movie soundtracks
- **Single + Live**: Live single releases

### Usage in Queries

```json
// Find only albums
{
  "entity": "release",
  "query": "artist:\"Pink Floyd\" AND type:Album"
}

// Find singles and EPs
{
  "entity": "release",
  "query": "artist:Radiohead AND (type:Single OR type:EP)"
}
```

---

## Artist Types

Categorizes artists by nature.

| Type | Description | Examples |
|------|-------------|----------|
| **Person** | Individual musician | David Bowie, Joni Mitchell, Jimi Hendrix |
| **Group** | Band or ensemble | The Beatles, Radiohead, Daft Punk |
| **Orchestra** | Classical orchestra | London Symphony Orchestra |
| **Choir** | Vocal ensemble | Mormon Tabernacle Choir |
| **Character** | Fictional character | Gorillaz characters, Hatsune Miku |
| **Other** | Doesn't fit categories | Various Artists, Unknown |

### Person-Specific Fields

For `type:Person`, additional fields available:
- **Gender**: Male, Female, Other, Not Applicable
- **Birth date**: Date of birth
- **Death date**: Date of death (if deceased)

### Usage in Queries

```json
// Find female solo artists
{
  "entity": "artist",
  "query": "type:Person AND gender:female"
}

// Find active groups
{
  "entity": "artist",
  "query": "type:Group AND end:*"  // No end date = still active
}
```

---

## Recordings vs Tracks

### Conceptual Difference

**Recording**:
- A unique performance/version of a song
- One recording can appear on many releases
- Has its own MBID

**Track**:
- A recording's appearance on a specific release
- Links a recording to a release with position info

### Example

**Recording**: "Bohemian Rhapsody" (studio version)
- MBID: `b1a9c0e9-d987-4042-ae91-78d6a3267d69`

**Tracks** (same recording on different releases):
1. "A Night at the Opera" (1975) - Track #11
2. "Greatest Hits" (1981) - Track #8
3. "Bohemian Rhapsody (single)" (1975) - Track #1

### Implications

When searching:
- Use [mb_recording_search](mb_recording_search.md) to find the unique recording
- Use `find_appearances: true` to see all tracks (releases)
- Recording MBID is stable; use it for tagging audio files

---

## Release vs Release Group

### Release

A **specific edition** of an album:
- 1997 UK CD pressing of "OK Computer"
- 2009 US vinyl reissue of "OK Computer"
- 2016 digital download of "OK Computer"

Each has its own:
- Release MBID
- Date, country, format
- Label, catalog number, barcode

### Release Group

The **abstract concept** of an album:
- "OK Computer" (the album itself)
- Groups all editions/pressings together

Has:
- Release Group MBID
- Primary type (Album, Single, EP)
- First release date

### Usage

**Use Release** when:
- Tagging specific audio files
- Identifying exact pressing/edition
- Getting tracklists

**Use Release Group** when:
- Researching album history
- Finding all editions
- General searches

---

## Date Formats

MusicBrainz uses flexible date formats.

### Formats Supported

| Format | Example | Precision |
|--------|---------|-----------|
| `YYYY-MM-DD` | `1997-05-21` | Exact date |
| `YYYY-MM` | `1997-05` | Month and year |
| `YYYY` | `1997` | Year only |

### Usage in Searches

```json
// Exact date
{"query": "date:1997-05-21"}

// Year only
{"query": "date:1997"}

// Date range
{"query": "date:[1990 TO 1999]"}
```

### Partial Dates

If only year is known, MusicBrainz stores as `YYYY` only.

---

## ISO Country Codes

Two-letter country codes per ISO 3166-1 alpha-2.

### Common Codes

| Code | Country |
|------|---------|
| `US` | United States |
| `GB` | United Kingdom |
| `FR` | France |
| `DE` | Germany |
| `JP` | Japan |
| `CA` | Canada |
| `AU` | Australia |
| `XW` | Worldwide |
| `XE` | Europe |

### Usage

```json
// Find UK releases
{"query": "artist:Radiohead AND country:GB"}

// Find US or UK releases
{"query": "artist:Beatles AND (country:US OR country:GB)"}
```

### Special Codes

- **XW**: Worldwide release (no specific country)
- **XE**: European release (multiple EU countries)
- **XU**: Unknown/unspecified

---

## Media Formats

Physical and digital formats for releases.

### Common Formats

| Format | Type | Description |
|--------|------|-------------|
| **CD** | Physical | Compact Disc |
| **Vinyl** | Physical | Vinyl record (LP, 7", 12") |
| **Digital Media** | Digital | Download or streaming |
| **Cassette** | Physical | Cassette tape |
| **DVD** | Physical | DVD (audio or video) |
| **Blu-ray** | Physical | Blu-ray disc |
| **USB** | Physical | USB flash drive |

### Usage

```json
// Find vinyl releases
{"query": "artist:\"Pink Floyd\" AND format:Vinyl"}

// Find digital releases
{"query": "release:\"Random Access Memories\" AND format:\"Digital Media\""}
```

---

## Tags

User-contributed genre and style labels.

### Nature

- Folksonomy (user-generated)
- Not official classifications
- Useful for discovery
- Vary in specificity

### Common Tags

- **Genres**: rock, jazz, electronic, classical, hip hop
- **Styles**: progressive rock, bebop, ambient, baroque
- **Moods**: melancholic, upbeat, atmospheric
- **Eras**: 80s, 90s, 2000s

### Usage

```json
// Find jazz artists
{"query": "tag:jazz"}

// Find electronic music from 2020-2023
{
  "entity": "release-group",
  "query": "tag:electronic AND date:[2020 TO 2023]"
}
```

### Caveats

- Not always accurate or consistent
- Popular artists have more/better tags
- Combine with other filters for best results

---

## Disambiguation

Additional text to distinguish similar entities.

### Purpose

Helps identify the correct entity when names are identical or similar.

### Examples

| Name | Disambiguation |
|------|----------------|
| Radiohead | UK rock band |
| The Who | English rock band |
| Michael Jackson | King of Pop |
| Michael Jackson | American R&B and funk songwriter |

### Display

Usually shown in parentheses:
```
Radiohead (UK rock band)
```

### When Used

Most commonly for:
- Common names with multiple artists
- Similar band names
- Artists with same/similar names

---

## See Also

- [Rate Limiting](rate-limiting.md) - API usage guidelines
- [Troubleshooting](troubleshooting.md) - Common issues
- Individual tool documentation for specific usage
