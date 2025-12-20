# Music Library Harmonization Guide for AI Agents

This guide provides essential instructions for AI agents tasked with organizing and harmonizing music libraries using the Music MCP Server.

---

## Mission

Harmonize audio file libraries by:
1. **Analyzing** existing metadata and file names
2. **Enriching** metadata from MusicBrainz (via search)
3. **Identifying** unknown files via fingerprinting (last resort)
4. **Renaming** files according to naming conventions
5. **Organizing** directory structure
6. **Downloading** cover art
7. **Cleaning** duplicate and unnecessary files

---

## Available Tools (12 Total)

### Filesystem (3)
- `fs_list_dir` - List directory contents (recursive support)
- `fs_rename` - Rename files/directories (with dry-run)
- `fs_delete` - Delete files/directories (permanent)

### Metadata (2)
- `read_metadata` - Read audio tags (MP3, FLAC, M4A, WAV, OGG)
- `write_metadata` - Write/update audio tags

### MusicBrainz (7)
- `mb_identify_record` - Audio fingerprinting via AcoustID
- `mb_artist_search` - Search artists, get releases
- `mb_release_search` - Search releases, get tracklists
- `mb_recording_search` - Search recordings
- `mb_work_search` - Search musical compositions
- `mb_label_search` - Search record labels
- `mb_cover_download` - Download album cover art

**⚠️ CRITICAL: Query Parameter Rules**
All MusicBrainz search tools have strict requirements for the `query` parameter:
- **Use ONLY the exact name/title** you're searching for
- **NEVER include** additional context (artist, year, format, etc.)
- The search engine does NOT work like natural language - extra words break it
- See examples below for each tool

---

## MusicBrainz Search: Correct Usage Examples

### ❌ COMMON MISTAKES TO AVOID

**DO NOT add contextual information to the query parameter:**

| Tool | ❌ WRONG | ✅ CORRECT | Why Wrong? |
|------|---------|-----------|-----------|
| `mb_artist_search` | "Radiohead UK band" | "Radiohead" | Extra words break artist search |
| `mb_artist_search` | "The Beatles 1960s" | "The Beatles" | Year is not part of artist name |
| `mb_release_search` | "OK Computer Radiohead" | "OK Computer" | Artist name breaks release search |
| `mb_release_search` | "Nevermind 1991" | "Nevermind" | Year breaks release search |
| `mb_release_search` | "Abbey Road CD" | "Abbey Road" | Format breaks search |
| `mb_recording_search` | "Imagine John Lennon" | "Imagine" | Artist name breaks recording search |
| `mb_recording_search` | "Smells Like Teen Spirit by Nirvana" | "Smells Like Teen Spirit" | "by X" breaks search |
| `mb_recording_search` | "Bohemian Rhapsody 1975" | "Bohemian Rhapsody" | Year breaks search |

### ✅ CORRECT WORKFLOW EXAMPLES

**Example 1: Finding metadata for "Imagine" by John Lennon**
```
Step 1: Search for the artist
  Tool: mb_artist_search
  Parameters:
    - search_type: "artist"
    - query: "John Lennon"  ✅ (NOT "John Lennon Beatles" ❌)

Step 2: Search for the recording
  Tool: mb_recording_search
  Parameters:
    - search_type: "recording"
    - query: "Imagine"  ✅ (NOT "Imagine John Lennon" ❌)

Step 3: Get releases containing the recording
  Tool: mb_recording_search
  Parameters:
    - search_type: "recording_releases"
    - query: "Imagine"  ✅ (or use the MBID from Step 2)
```

**Example 2: Finding all tracks on "OK Computer" by Radiohead**
```
Step 1: Search for the release
  Tool: mb_release_search
  Parameters:
    - search_type: "release"
    - query: "OK Computer"  ✅ (NOT "OK Computer Radiohead 1997" ❌)

Step 2: Get tracklist
  Tool: mb_release_search
  Parameters:
    - search_type: "release_recordings"
    - query: "<MBID from Step 1>"  ✅ (or "OK Computer")
```

**Example 3: Working with metadata from filename "Nirvana - Nevermind - 03 - Smells Like Teen Spirit.mp3"**
```
Extracted information:
  - Artist: "Nirvana"
  - Album: "Nevermind"
  - Track: "Smells Like Teen Spirit"

Step 1: Search artist
  Tool: mb_artist_search
  Parameters:
    - search_type: "artist"
    - query: "Nirvana"  ✅ (extract ONLY the artist name)

Step 2: Search release
  Tool: mb_release_search
  Parameters:
    - search_type: "release"
    - query: "Nevermind"  ✅ (extract ONLY the album name)

Step 3: Search recording
  Tool: mb_recording_search
  Parameters:
    - search_type: "recording"
    - query: "Smells Like Teen Spirit"  ✅ (extract ONLY the track title)
```

---

## Standard Workflow

### Phase 1: Discovery & Analysis
```
1. fs_list_dir          → Scan music directory (recursive)
2. read_metadata        → Check existing tags for each file
3. Analyze              → Identify files needing correction
```

### Phase 2: Identification & Enrichment
```
For files with usable metadata/filenames:
4. mb_artist_search     → Search by artist name ONLY (extract just the name)
5. mb_release_search    → Search by album/release name ONLY (extract just the title)
6. mb_recording_search  → Search by track title ONLY (extract just the title)

⚠️ CRITICAL: When extracting data from filenames or existing metadata:
   - Parse/split the information first
   - Use ONLY the relevant part for each search
   - Example: From "Beatles - Abbey Road - 01 - Come Together.mp3"
     * For artist search: use "Beatles" (not the whole filename)
     * For release search: use "Abbey Road" (not the whole filename)
     * For recording search: use "Come Together" (not the whole filename)

For completely unknown files (LAST RESORT):
7. mb_identify_record   → Audio fingerprinting (requires fpcalc)

Then:
8. write_metadata       → Update file tags with correct data
9. mb_cover_download    → Download missing cover art
```

### Phase 3: Organization
```
10. fs_rename (dry-run) → Preview renaming (ALWAYS test first)
11. fs_rename           → Apply renaming
12. read_metadata       → Verify changes
```

### Phase 4: Cleanup
```
13. fs_delete           → Remove duplicates/unnecessary files
```

---

## Naming Convention

*TODO: To be filled manually by the user*

### Directory Structure

Directory path: /mnt/data/test

*TODO: To be filled manually by the user*


### File Naming Pattern

*TODO: To be filled manually by the user*

---

## Critical Safety Rules

### ALWAYS
✅ Use `dry_run: true` with `fs_rename` before actual rename
✅ Read metadata before writing
✅ List directory before deleting
✅ Verify changes after operations
✅ Handle errors gracefully
✅ **Extract and parse** filename/metadata components before searching
✅ **Use ONLY the exact name/title** in MusicBrainz query parameters

### NEVER
❌ Delete without confirming content first
❌ Rename without dry-run test
❌ Overwrite tags without reading existing ones
❌ Process files outside configured root directory
❌ Use wildcards (not supported)
❌ **Pass entire filenames or compound strings to MusicBrainz search tools**
❌ **Add artist names when searching for releases/recordings**
❌ **Add album names when searching for artists/recordings**
❌ **Add years, formats, or other context to search queries**

---

## Metadata Fields Reference

### Standard Tags
| Field | Type | Example |
|-------|------|---------|
| `title` | string | "Bohemian Rhapsody" |
| `artist` | string | "Queen" |
| `album` | string | "A Night at the Opera" |
| `album_artist` | string | "Queen" |
| `year` | integer | 1975 |
| `track` | integer | 11 |
| `track_total` | integer | 12 |
| `genre` | string | "Progressive Rock" |
| `comment` | string | "Remastered 2011" |

### Technical Properties (read-only)
- `duration_seconds`, `bitrate_kbps`, `sample_rate_hz`, `channels`, `bit_depth`

---

## Error Handling

### Common Issues
| Error | Cause | Solution |
|-------|-------|----------|
| "Path security validation failed" | File outside root | Check MCP_ROOT_PATH |
| "Failed to read audio file" | Unsupported/corrupted | Verify format |
| "No matches found" | Poor fingerprint | Try manual search |
| "Rate limit exceeded" | Too many MB requests | Wait 1 second between calls |
| **"No artists/releases/recordings found"** | **Query contains extra info** | **Extract ONLY the name/title** |

### MusicBrainz Search Debugging

If you get "No results found" from MusicBrainz searches:

1. **Check your query parameter:**
   - Does it contain ONLY the artist name / album title / track title?
   - Remove any artist names from album/track searches
   - Remove any years, formats, or descriptive text
   - Remove phrases like "by", "from", "feat.", etc.

2. **Example debugging:**
   ```
   ❌ Query: "Imagine John Lennon"
   ✅ Fixed: "Imagine"

   ❌ Query: "OK Computer Radiohead 1997"
   ✅ Fixed: "OK Computer"

   ❌ Query: "Nirvana grunge band"
   ✅ Fixed: "Nirvana"
   ```

3. **If still no results:**
   - Verify spelling
   - Try alternative titles (e.g., "The Beatles" vs "Beatles")
   - Check if it's a very obscure release
   - As last resort, use `mb_identify_record` with audio fingerprinting

### Response Format
- Success: `isError: false`, structured content
- Error: `isError: true`, text description

---

## Performance Tips

1. **Batch operations**: Process files sequentially, not in parallel
2. **Rate limiting**: MusicBrainz allows 1 request/second
3. **Metadata level**: Use "Minimal" for fingerprinting unless full data needed
4. **Recursive depth**: Limit `fs_list_dir` depth to 2-3 for large libraries
5. **Properties**: Only include technical properties when analyzing quality

---

## Quick Reference

### Supported Audio Formats
✅ MP3, FLAC, M4A/AAC, OGG/Vorbis, Opus, WAV, AIFF

### Not Supported
❌ Album artwork in tags (use `mb_cover_download` to separate file)
❌ Lyrics, ReplayGain, custom tags
❌ Wildcard patterns in file operations

---

**Remember**: Safety first. Always dry-run, always verify, always preserve original audio data.
