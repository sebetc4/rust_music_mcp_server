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
4. mb_artist_search     → Search by artist name
5. mb_release_search    → Search by album/release name
6. mb_recording_search  → Search by track title

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

### NEVER
❌ Delete without confirming content first
❌ Rename without dry-run test
❌ Overwrite tags without reading existing ones
❌ Process files outside configured root directory
❌ Use wildcards (not supported)

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
