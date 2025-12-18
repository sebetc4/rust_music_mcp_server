# MusicBrainz Tools

Comprehensive documentation for all MusicBrainz tools available in the Music MCP Server.

---

## Overview

The MusicBrainz tools provide integration with the [MusicBrainz](https://musicbrainz.org/) database and [AcoustID](https://acoustid.org/) fingerprinting service for music metadata discovery, enrichment, and audio file identification.

### What is MusicBrainz?

MusicBrainz is an open music encyclopedia that collects music metadata and makes it available to the public. It contains information about:

- **Artists**: Musicians, bands, composers
- **Releases**: Albums, singles, compilations
- **Recordings**: Individual tracks
- **Works**: Musical compositions
- **Labels**: Record labels

### What is AcoustID?

AcoustID is an acoustic fingerprinting system that can identify audio files by analyzing their acoustic properties using the Chromaprint algorithm.

---

## Available Tools

### Search Tools

| Tool | Purpose | Documentation |
|------|---------|---------------|
| **mb_artist_search** | Search for artists and get their releases | [→ Documentation](mb/mb_artist_search.md) |
| **mb_release_search** | Search for releases and get tracklists | [→ Documentation](mb/mb_release_search.md) |
| **mb_recording_search** | Search for recordings and find appearances | [→ Documentation](mb/mb_recording_search.md) |
| **mb_advanced_search** | Complex queries with Lucene syntax | [→ Documentation](mb/mb_advanced_search.md) |

### Identification Tools

| Tool | Purpose | Documentation |
|------|---------|---------------|
| **mb_identify_record** | Identify audio files via fingerprinting | [→ Documentation](mb/mb_identify_record.md) |

---

## Tool Comparison

### When to Use Each Tool

#### mb_artist_search
**Use when you need to**:
- Find an artist's MBID
- Discover an artist's discography
- Verify artist information (country, type, active period)
- Explore artists with similar names

**Example use cases**:
- "Find all albums by Radiohead"
- "Get the MBID for Daft Punk"
- "List all releases by Miles Davis"

[→ Full documentation](mb/mb_artist_search.md)

---

#### mb_release_search
**Use when you need to**:
- Find specific album/single information
- Get complete tracklists with recording MBIDs
- Compare different editions (remasters, countries)
- Verify release dates and labels

**Example use cases**:
- "Get the tracklist for OK Computer"
- "Find all versions of Discovery by Daft Punk"
- "Get metadata for this album barcode"

[→ Full documentation](mb/mb_release_search.md)

---

#### mb_recording_search
**Use when you need to**:
- Search for individual tracks
- Find which releases contain a recording
- Discover live versions or covers
- Track down rare releases

**Example use cases**:
- "Which albums contain Paranoid Android?"
- "Find all releases with this track"
- "Get recording MBID for this song"

[→ Full documentation](mb/mb_recording_search.md)

---

#### mb_advanced_search
**Use when you need to**:
- Complex multi-criteria searches
- Filter by fields not in simple search
- Find releases meeting exact requirements
- Use boolean logic and ranges

**Example use cases**:
- "Find UK vinyl releases from 1970-1980"
- "Search for jazz artists from France"
- "Find albums with 12 tracks released in 2020"

[→ Full documentation](mb/mb_advanced_search.md)

---

#### mb_identify_record
**Use when you need to**:
- Identify unknown music files
- Auto-tag music libraries
- Verify file identity against expected tracks
- Discover official release info for any audio

**Example use cases**:
- "What song is this MP3 file?"
- "Tag my entire music collection"
- "Is this file really what the filename says?"

**Requirements**: fpcalc binary (Chromaprint) must be installed

[→ Full documentation](mb/mb_identify_record.md)

---

## Quick Start Examples

### Example 1: Find Artist and Get Discography

```json
// Step 1: Search for artist
{
  "name": "mb_artist_search",
  "arguments": {
    "artist": "Radiohead",
    "include_releases": true
  }
}

// Returns artist MBID and list of releases
```

---

### Example 2: Get Album Tracklist

```json
// Step 1: Find the release
{
  "name": "mb_release_search",
  "arguments": {
    "release": "OK Computer",
    "artist": "Radiohead"
  }
}

// Step 2: Get full tracklist using release MBID
{
  "name": "mb_release_search",
  "arguments": {
    "release_mbid": "52709206-8816-3c12-9ff6-f957f2f1eecf",
    "include_tracklist": true
  }
}
```

---

### Example 3: Identify Unknown Audio File

```json
{
  "name": "mb_identify_record",
  "arguments": {
    "file_path": "/music/unknown_track.mp3",
    "metadata_level": "basic"
  }
}

// Returns recording, artist, and release information
```

---

### Example 4: Advanced Search

```json
{
  "name": "mb_advanced_search",
  "arguments": {
    "entity": "release",
    "query": "artist:\"Pink Floyd\" AND format:Vinyl AND date:[1970 TO 1980]",
    "limit": 25
  }
}

// Returns Pink Floyd vinyl releases from the 1970s
```

---

## Common Workflows

### Workflow 1: Complete Artist Metadata Collection

```
1. mb_artist_search (artist: "name", include_releases: true)
   → Get artist MBID and all release MBIDs

2. For each release MBID:
   mb_release_search (release_mbid: "...", include_tracklist: true)
   → Get complete tracklist with recording MBIDs

3. For interesting recordings:
   mb_recording_search (recording_mbid: "...", find_appearances: true)
   → Find other releases containing the track
```

---

### Workflow 2: Audio File Identification and Tagging

```
1. mb_identify_record (file_path: "/music/unknown.mp3", metadata_level: "full")
   → Get recording, artist, release, and complete metadata

2. If no match found:
   mb_recording_search (recording: "suspected name", artist: "suspected artist")
   → Manual search as fallback

3. Apply metadata to file using write_metadata tool
```

---

### Workflow 3: Find Best Release Version

```
1. mb_release_search (release: "album name", artist: "artist name")
   → Get initial results

2. mb_release_search (release: "album name", artist: "artist name",
                      find_release_group_versions: true)
   → Find all editions/versions

3. Compare:
   - Release dates
   - Countries
   - Formats (CD, Vinyl, Digital)
   - Track counts
   - Labels

4. Select preferred version and get full tracklist
```

---

## Shared Concepts

Concepts common to all tools:

- **[Common Concepts](mb/common-concepts.md)** - MBIDs, entity types, release status, etc.
- **[Rate Limiting](mb/rate-limiting.md)** - API usage guidelines and best practices
- **[Troubleshooting](mb/troubleshooting.md)** - Common issues and solutions

---

## Requirements

### Network Access
All tools require internet access to:
- MusicBrainz API: https://musicbrainz.org/
- AcoustID API: https://acoustid.org/ (for identification)

### System Requirements
- **mb_identify_record only**: Requires `fpcalc` binary (Chromaprint package)
  - Ubuntu/Debian: `sudo apt-get install libchromaprint-tools`
  - macOS: `brew install chromaprint`
  - Windows: Download from https://acoustid.org/chromaprint

---

## Rate Limiting

### MusicBrainz API
- **Limit**: 1 request per second
- **Enforcement**: Server-side (HTTP 429)
- **Handling**: Automatic in all tools

### AcoustID API
- **Limit**: Fair use (~3 requests/second sustained)
- **Best practice**: Add delays for large batches

### Testing
Always run network tests sequentially:
```bash
cargo test -- --ignored --test-threads=1
```

See [Rate Limiting](mb/rate-limiting.md) for detailed guidelines.

---

## Architecture Alignment

This documentation structure mirrors the code architecture:

```
src/domains/tools/definitions/mb/     documentation/tools/mb/
├── artist.rs                    →    ├── mb_artist_search.md
├── release.rs                   →    ├── mb_release_search.md
├── recording.rs                 →    ├── mb_recording_search.md
├── advanced.rs                  →    ├── mb_advanced_search.md
├── identify_record.rs           →    ├── mb_identify_record.md
├── common.rs                    →    ├── common-concepts.md
└── mod.rs                       →    └── (shared docs)
```

This alignment makes it easy to:
- Update documentation when code changes
- Find relevant docs for each tool
- Maintain consistency between code and docs

---

## External Resources

- **MusicBrainz Website**: https://musicbrainz.org/
- **MusicBrainz API Documentation**: https://musicbrainz.org/doc/MusicBrainz_API
- **AcoustID Website**: https://acoustid.org/
- **Chromaprint**: https://acoustid.org/chromaprint
- **Lucene Query Syntax**: https://lucene.apache.org/core/2_9_4/queryparsersyntax.html

---

## Related Documentation

- [Tool System Architecture](../architecture/tool-system.md) - How tools work internally
- [External APIs](../architecture/external-apis.md) - Integration details
- [Adding New Tools](../guides/adding-tools.md) - Extending with new tools
- [Error Handling](../reference/error-handling.md) - Error patterns and handling

---

## Quick Navigation

### By Task

- **Search for an artist** → [mb_artist_search](mb/mb_artist_search.md)
- **Search for an album** → [mb_release_search](mb/mb_release_search.md)
- **Search for a track** → [mb_recording_search](mb/mb_recording_search.md)
- **Complex search query** → [mb_advanced_search](mb/mb_advanced_search.md)
- **Identify audio file** → [mb_identify_record](mb/mb_identify_record.md)

### By Topic

- **Understanding MBIDs** → [Common Concepts](mb/common-concepts.md#musicbrainz-identifiers-mbids)
- **Release types explained** → [Common Concepts](mb/common-concepts.md#release-types)
- **Rate limit guidelines** → [Rate Limiting](mb/rate-limiting.md)
- **No results found** → [Troubleshooting](mb/troubleshooting.md#no-results-found)
- **fpcalc not found** → [Troubleshooting](mb/troubleshooting.md#fpcalc-not-found)

---

## Contributing to Documentation

When updating tools:

1. **Update code** in `src/domains/tools/definitions/mb/`
2. **Update corresponding doc** in `documentation/tools/mb/`
3. **Keep examples current** - test examples still work
4. **Update this index** if adding/removing tools

This ensures documentation stays synchronized with implementation.
