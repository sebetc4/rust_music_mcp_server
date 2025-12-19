# MCP Prompts & Resources Specification

This document defines prompts and resources to be exposed via the MCP protocol for enhanced LLM context and guidance.

---

## Overview

**Purpose**: Provide structured context to MCP clients (LLMs) for better decision-making and workflow guidance.

**Scope**: These are NOT code documentation - they are runtime resources exposed through MCP's `prompts/list` and `resources/list` endpoints.

---

## Prompts

Prompts are pre-defined workflows that guide the LLM through complex multi-step tasks.

### 1. Workflow Orchestration Prompts

#### `organize_album`
**Description**: Complete album organization workflow from scan to verification

**Context Provided**:
```markdown
WORKFLOW: Organize Album
1. Scan directory with fs_list_dir
2. Read metadata for all tracks
3. Strategy decision:
   - Has artist/album in tags? → Use mb_release_search
   - Has filename pattern? → Parse and search
   - No info? → Use mb_identify_record (last resort)
4. Get complete metadata from MusicBrainz
5. Update all track tags with write_metadata
6. Download cover art with mb_cover_download
7. Rename files (DRY RUN first!)
8. Apply rename
9. Verify with read_metadata

SAFETY: Always dry-run, preserve audio data, handle errors gracefully
```

**Arguments**:
- `album_path` (required): Path to album directory

---

#### `clean_library`
**Description**: Comprehensive library cleanup and organization

**Context Provided**:
```markdown
WORKFLOW: Clean Music Library
1. Deep scan with fs_list_dir (recursive)
2. Categorize files:
   - Complete metadata → Ready for organization
   - Incomplete metadata → Needs enrichment
   - No metadata → Needs identification
   - Duplicates → Needs deduplication
3. For each category, apply appropriate workflow
4. Generate cleanup report

FOCUS: Duplicates, incomplete tags, naming inconsistencies
```

**Arguments**:
- `library_path` (required): Root library path
- `scope` (optional): "metadata_only" | "full_cleanup" | "duplicates_only"

---

#### `identify_unknowns`
**Description**: Batch identification of files with missing metadata

**Context Provided**:
```markdown
WORKFLOW: Identify Unknown Files
1. Find files with missing title OR artist
2. For each file:
   a. Check filename for patterns (e.g., "Artist - Title.mp3")
   b. If parseable → Use mb_recording_search
   c. If not → Use mb_identify_record (fingerprint)
3. Update metadata
4. Report: identified count, failed count, manual intervention needed

EFFICIENCY: Prefer search over fingerprint (1 req/sec rate limit)
```

**Arguments**:
- `search_path` (required): Directory to scan
- `max_fingerprints` (optional): Limit fingerprint operations (default: 50)

---

### 2. Decision Support Prompts

#### `suggest_strategy`
**Description**: Analyze file and recommend identification method

**Context Provided**:
```markdown
DECISION TREE: File Identification Strategy

Input: File path
Process:
1. Read metadata with read_metadata
2. Analyze:
   - title + artist present? → SEARCH (mb_recording_search)
   - album + artist present? → SEARCH (mb_release_search)
   - only filename available? → PARSE then SEARCH
   - nothing usable? → FINGERPRINT (mb_identify_record)

Output: Recommended tool + query/path + reasoning

COST ANALYSIS:
- Search: Free, fast, 1 req/sec
- Fingerprint: Requires fpcalc, slower, 1 req/sec, less accurate for live/remixes
```

**Arguments**:
- `file_path` (required): Audio file to analyze

---

#### `compare_releases`
**Description**: Help choose between multiple MusicBrainz releases

**Context Provided**:
```markdown
DECISION: Release Selection Criteria

When mb_release_search returns multiple matches:

PRIORITY ORDER:
1. **Country**: User's region > Original release country > International
2. **Date**: Original release > Remaster (unless remaster explicitly wanted)
3. **Format**: Digital > CD > Vinyl (for tagging purposes)
4. **Status**: Official > Promotion > Bootleg
5. **Label**: Major label > Independent (if quality matters)

SPECIAL CASES:
- Deluxe editions: Extra tracks, usually preferred
- Remasters: Check year in existing metadata
- Compilations: Use "Various Artists" as album_artist

OUTPUT: Recommend release MBID with reasoning
```

**Arguments**:
- `releases` (required): JSON array of release candidates
- `existing_metadata` (optional): Current file metadata for comparison

---

#### `validate_metadata`
**Description**: Check metadata quality and completeness

**Context Provided**:
```markdown
VALIDATION: Metadata Quality Checklist

REQUIRED FIELDS (must be present):
- title
- artist
- album

RECOMMENDED FIELDS (should be present):
- album_artist
- year
- track (if part of album)
- genre

QUALITY CHECKS:
- Title case consistency
- "feat." vs featuring tag
- Album artist for compilations
- Track numbering sequence
- Year format (4 digits)

OUTPUT: Pass/fail + missing fields + suggestions
```

**Arguments**:
- `metadata` (required): JSON object with current tags

---

### 3. Diagnostic Prompts

#### `troubleshoot_identification`
**Description**: Debug failed identification attempts

**Context Provided**:
```markdown
TROUBLESHOOTING: Identification Failure

COMMON CAUSES:
1. **No search results**:
   - Query too specific (try removing year/extra words)
   - Misspelled artist/title
   - Non-mainstream/local music not in MusicBrainz

2. **Fingerprint failed**:
   - File corrupted
   - Format not supported by fpcalc
   - Live recording (fingerprints unreliable)
   - Very short track (< 30 seconds)

3. **Multiple ambiguous results**:
   - Need more context (album name helps)
   - Try mb_release_search instead of recording

FALLBACK ACTIONS:
- Manual search on musicbrainz.org
- Use partial metadata update
- Ask user for confirmation

OUTPUT: Diagnosis + recommended next steps
```

**Arguments**:
- `file_path` (required): Problem file
- `attempted_method` (required): "search" | "fingerprint"
- `error_message` (optional): Error received

---

### 4. Educational Prompts

#### `explain_musicbrainz_structure`
**Description**: MusicBrainz data model explanation

**Context Provided**:
```markdown
REFERENCE: MusicBrainz Entity Hierarchy

ARTIST
  └─ RELEASE GROUP (conceptual album)
      └─ RELEASE (specific edition)
          └─ MEDIUM (disc/LP)
              └─ TRACK
                  └─ RECORDING (actual audio)

EXAMPLE:
Artist: "Pink Floyd"
Release Group: "The Wall" (album concept)
Release: "The Wall (1979, US, Columbia)" (specific CD)
Medium: "Disc 1"
Track: "Track 3: Another Brick in the Wall, Part 1"
Recording: "Another Brick in the Wall, Part 1 (1979 studio)"

WHY IT MATTERS:
- mb_release_search can find release or release group
- Use release MBID (not release group) for accurate tracklist
- Same recording can appear on multiple releases

KEY FIELDS:
- MBID: Unique identifier (use for exact lookups)
- Release Status: Official | Promotion | Bootleg
- Release Country: Different releases per region
```

---

#### `metadata_best_practices`
**Description**: Genre-specific tagging guidelines

**Context Provided**:
```markdown
BEST PRACTICES: Metadata Tagging by Genre

CLASSICAL MUSIC:
- Artist = Performer/Orchestra
- Album Artist = Composer
- Title = "Work: Movement" (e.g., "Symphony No. 5: I. Allegro")
- Use mb_work_search for compositions

COMPILATIONS:
- Album Artist = "Various Artists"
- Artist = Individual track artist
- Comment = "Compilation" (optional)

SOUNDTRACKS:
- Album Artist = "Soundtrack" or composer name
- Artist = Performing artist per track
- Album = "[Movie/Game Name] (Original Soundtrack)"

LIVE RECORDINGS:
- Title should include "(Live)"
- Year = Recording year (not release year)
- Comment = Venue/date info

FEATURING ARTISTS:
- Title: "Song Title (feat. Artist Name)"
- Do NOT put featuring artist in main artist field
- Some formats support separate "featured artist" tag

GENERAL RULES:
- Use title case consistently
- Avoid ALL CAPS or all lowercase
- Year = Original release year (or recording year if unreleased)
- Genre = Broad category (avoid micro-genres)
```

**Arguments**:
- `genre` (optional): Specific genre to focus on

---

### 5. Specialized Prompts

#### `classical_music_tagging`
**Description**: Detailed classical music metadata strategy

**Context Provided**:
```markdown
SPECIALIZED: Classical Music Tagging

CHALLENGES:
- Multiple artists (composer, conductor, orchestra, soloists)
- Complex work structures (symphony > movement)
- Catalog numbers (BWV, K., Op.)

RECOMMENDED STRUCTURE:
- Title: "[Catalog] Work Name: Movement"
  Example: "BWV 1007 Cello Suite No. 1: I. Prelude"
- Artist: Primary performer
- Album Artist: Composer
- Album: "Composer: Work(s)"
- Comment: Conductor, orchestra, date

TOOLS TO USE:
- mb_work_search: Find composition MBID
- mb_recording_search: Find specific performance
- mb_release_search: Find complete album

NAMING CONVENTION:
{Composer}/{Catalog} {Work Name}/{Track} - {Movement}.ext
Example: Bach/BWV 1007 Cello Suite No. 1/01 - I. Prelude.flac
```

---

#### `compilation_handling`
**Description**: Multi-artist album organization

**Context Provided**:
```markdown
SPECIALIZED: Compilation Albums

DEFINITION: Album with tracks from different artists

METADATA REQUIREMENTS:
- Album Artist: "Various Artists" (CRITICAL)
- Artist: Individual per track
- Title: Track title (no artist prefix needed)

DIRECTORY STRUCTURE:
Option 1: Various Artists/{Album Name} ({Year})
Option 2: Compilations/{Album Name} ({Year})

FILE NAMING:
{track:02d} - {artist} - {title}.ext
Example: 01 - Queen - Bohemian Rhapsody.flac

MUSICBRAINZ SEARCH:
- Look for release with "compilation" in type
- Verify tracklist matches (compilations vary by region)

COMMON MISTAKES TO AVOID:
❌ Using first artist as album artist
❌ Putting "Various Artists" in track artist field
❌ Organizing under artist folders
```

---

## Resources

Resources are static/dynamic data exposed to provide reference information and decision support.

### 1. Configuration Resources

#### `naming_schemes`
**URI**: `music://config/naming-schemes`

**Description**: Standard file/directory naming patterns

**Content**:
```json
{
  "schemes": [
    {
      "id": "plex",
      "name": "Plex Media Server",
      "directory": "{album_artist}/{album} ({year})",
      "file": "{track:02d} - {title}",
      "compilation_directory": "Various Artists/{album} ({year})",
      "notes": "Optimized for Plex auto-detection"
    },
    {
      "id": "audiophile",
      "name": "Audiophile Detailed",
      "directory": "{album_artist}/{year} - {album} [{format} {bitrate}kbps]",
      "file": "{track:02d}. {artist} - {title}",
      "notes": "Includes quality info in folder name"
    },
    {
      "id": "genre_based",
      "name": "Genre Organization",
      "directory": "{genre}/{album_artist}/{album} ({year})",
      "file": "{track:02d} - {title}",
      "notes": "Organized by genre first"
    },
    {
      "id": "minimal",
      "name": "Minimal Clean",
      "directory": "{album_artist}/{album}",
      "file": "{track:02d} {title}",
      "notes": "Simple, no year/extra info"
    }
  ],
  "variables": {
    "album_artist": "Album artist name",
    "artist": "Track artist name",
    "album": "Album name",
    "title": "Track title",
    "year": "Release year (4 digits)",
    "track": "Track number (integer)",
    "genre": "Music genre",
    "format": "Audio format (FLAC, MP3, etc.)",
    "bitrate": "Bitrate in kbps"
  }
}
```

---

#### `genre_taxonomy`
**URI**: `music://config/genre-taxonomy`

**Description**: Standardized genre list with MusicBrainz alignment

**Content**:
```json
{
  "genres": [
    {
      "name": "Rock",
      "mb_tag": "rock",
      "subgenres": ["Alternative Rock", "Indie Rock", "Progressive Rock", "Hard Rock"],
      "description": "Guitar-driven popular music"
    },
    {
      "name": "Electronic",
      "mb_tag": "electronic",
      "subgenres": ["House", "Techno", "Ambient", "Drum and Bass"],
      "description": "Synthesizer and computer-based music"
    },
    {
      "name": "Jazz",
      "mb_tag": "jazz",
      "subgenres": ["Bebop", "Fusion", "Free Jazz", "Smooth Jazz"],
      "description": "Improvisation-based music"
    },
    {
      "name": "Classical",
      "mb_tag": "classical",
      "subgenres": ["Baroque", "Romantic", "Contemporary Classical"],
      "description": "Western art music tradition"
    },
    {
      "name": "Hip Hop",
      "mb_tag": "hip hop",
      "subgenres": ["Rap", "Trap", "Conscious Hip Hop"],
      "description": "Rhythmic vocal delivery over beats"
    }
  ],
  "notes": "Use broad categories. Avoid micro-genres for consistency."
}
```

---

### 2. Reference Resources

#### `audio_format_guide`
**URI**: `music://reference/audio-formats`

**Description**: Audio format comparison and recommendations

**Content**:
```json
{
  "formats": {
    "lossless": [
      {
        "name": "FLAC",
        "extension": ".flac",
        "tag_system": "Vorbis Comments",
        "pros": ["No quality loss", "Good compression", "Wide support"],
        "cons": ["Larger file size"],
        "recommended_use": "Archival, high-quality playback",
        "typical_bitrate": "~1000 kbps"
      },
      {
        "name": "WAV",
        "extension": ".wav",
        "tag_system": "RIFF INFO / ID3",
        "pros": ["Universal support", "No compression overhead"],
        "cons": ["Very large files", "Limited metadata"],
        "recommended_use": "Professional audio work",
        "typical_bitrate": "~1411 kbps (16-bit/44.1kHz)"
      }
    ],
    "lossy": [
      {
        "name": "MP3",
        "extension": ".mp3",
        "tag_system": "ID3v2",
        "pros": ["Universal support", "Small size", "Mature"],
        "cons": ["Quality loss", "Older codec"],
        "recommended_use": "Portable devices, streaming",
        "recommended_bitrate": "≥ 320 kbps for transparency"
      },
      {
        "name": "AAC/M4A",
        "extension": ".m4a",
        "tag_system": "MP4/iTunes",
        "pros": ["Better quality than MP3", "Apple ecosystem"],
        "cons": ["Some compatibility issues"],
        "recommended_use": "Apple devices, modern streaming",
        "recommended_bitrate": "≥ 256 kbps"
      },
      {
        "name": "Opus",
        "extension": ".opus",
        "tag_system": "Vorbis Comments",
        "pros": ["Best quality/size ratio", "Modern codec"],
        "cons": ["Limited device support"],
        "recommended_use": "Streaming, voice",
        "recommended_bitrate": "≥ 128 kbps"
      }
    ]
  },
  "recommendations": {
    "archival": "FLAC",
    "everyday_listening": "MP3 320kbps or AAC 256kbps",
    "portable": "MP3 256kbps",
    "streaming": "Opus 128kbps"
  }
}
```

---

#### `metadata_field_mapping`
**URI**: `music://reference/metadata-mapping`

**Description**: Cross-format metadata field compatibility

**Content**:
```json
{
  "fields": [
    {
      "standard": "title",
      "id3v2": "TIT2",
      "vorbis": "TITLE",
      "mp4": "©nam",
      "description": "Track title",
      "support": ["MP3", "FLAC", "M4A", "OGG", "OPUS", "WAV"]
    },
    {
      "standard": "artist",
      "id3v2": "TPE1",
      "vorbis": "ARTIST",
      "mp4": "©ART",
      "description": "Track artist/performer",
      "support": ["MP3", "FLAC", "M4A", "OGG", "OPUS", "WAV"]
    },
    {
      "standard": "album",
      "id3v2": "TALB",
      "vorbis": "ALBUM",
      "mp4": "©alb",
      "description": "Album name",
      "support": ["MP3", "FLAC", "M4A", "OGG", "OPUS", "WAV"]
    },
    {
      "standard": "album_artist",
      "id3v2": "TPE2",
      "vorbis": "ALBUMARTIST",
      "mp4": "aART",
      "description": "Album artist (crucial for compilations)",
      "support": ["MP3", "FLAC", "M4A", "OGG", "OPUS"]
    },
    {
      "standard": "year",
      "id3v2": "TDRC",
      "vorbis": "DATE",
      "mp4": "©day",
      "description": "Release year (YYYY format)",
      "support": ["MP3", "FLAC", "M4A", "OGG", "OPUS", "WAV"]
    },
    {
      "standard": "track",
      "id3v2": "TRCK",
      "vorbis": "TRACKNUMBER",
      "mp4": "trkn",
      "description": "Track number",
      "support": ["MP3", "FLAC", "M4A", "OGG", "OPUS"]
    },
    {
      "standard": "genre",
      "id3v2": "TCON",
      "vorbis": "GENRE",
      "mp4": "©gen",
      "description": "Music genre",
      "support": ["MP3", "FLAC", "M4A", "OGG", "OPUS", "WAV"]
    }
  ],
  "notes": {
    "wav_support": "WAV metadata support varies by player. FLAC preferred for lossless.",
    "id3v2_version": "Always write ID3v2.4 for MP3 (used by write_metadata tool)",
    "compilation_flag": "Use album_artist='Various Artists' instead of compilation flag"
  }
}
```

---

### 3. Decision Support Resources

#### `fingerprint_decision_tree`
**URI**: `music://decision/fingerprint-strategy`

**Description**: When to use audio fingerprinting

**Content**:
```json
{
  "decision_flow": {
    "start": "check_metadata",
    "nodes": {
      "check_metadata": {
        "question": "Does file have title AND artist metadata?",
        "yes": "use_search",
        "no": "check_filename"
      },
      "check_filename": {
        "question": "Can filename be parsed for artist/title?",
        "yes": "parse_and_search",
        "no": "check_album"
      },
      "check_album": {
        "question": "Does file have album metadata?",
        "yes": "search_by_album",
        "no": "use_fingerprint"
      },
      "use_search": {
        "action": "mb_recording_search",
        "cost": "Low",
        "accuracy": "High (90%+)",
        "note": "Preferred method"
      },
      "parse_and_search": {
        "action": "Parse filename → mb_recording_search",
        "cost": "Low",
        "accuracy": "Medium (70-80%)",
        "note": "Depends on filename quality"
      },
      "search_by_album": {
        "action": "mb_release_search",
        "cost": "Low",
        "accuracy": "Medium (requires manual track matching)",
        "note": "Good for album batches"
      },
      "use_fingerprint": {
        "action": "mb_identify_record",
        "cost": "High (requires fpcalc, slower)",
        "accuracy": "Medium (60-80%, fails on live/remixes)",
        "note": "LAST RESORT"
      }
    }
  },
  "cost_comparison": {
    "search": {
      "time": "~200ms",
      "dependencies": "None",
      "rate_limit": "1 req/sec",
      "reliability": "High"
    },
    "fingerprint": {
      "time": "~2-5 seconds",
      "dependencies": "fpcalc binary required",
      "rate_limit": "1 req/sec (AcoustID API)",
      "reliability": "Medium (fails on: live recordings, very short tracks, heavy compression)"
    }
  }
}
```

---

#### `duplicate_detection_rules`
**URI**: `music://decision/duplicate-detection`

**Description**: Criteria for identifying duplicate files

**Content**:
```json
{
  "rules": [
    {
      "priority": 1,
      "name": "Exact Match (MusicBrainz ID)",
      "criteria": {
        "mbid": "Same recording MBID in metadata"
      },
      "confidence": "100%",
      "action": "Compare bitrate/format, keep higher quality"
    },
    {
      "priority": 2,
      "name": "Metadata Match",
      "criteria": {
        "title": "Exact match (case-insensitive)",
        "artist": "Exact match (case-insensitive)",
        "album": "Exact match (case-insensitive)",
        "duration_diff": "< 2 seconds"
      },
      "confidence": "95%",
      "action": "Compare technical properties"
    },
    {
      "priority": 3,
      "name": "Fingerprint Match",
      "criteria": {
        "acoustid_fingerprint": "Same fingerprint"
      },
      "confidence": "90%",
      "action": "Different releases of same recording, keep preferred format"
    },
    {
      "priority": 4,
      "name": "Fuzzy Match",
      "criteria": {
        "title": "Similar (Levenshtein distance < 3)",
        "artist": "Exact match",
        "duration_diff": "< 5 seconds"
      },
      "confidence": "70%",
      "action": "Flag for manual review"
    }
  ],
  "quality_preference": [
    {
      "rank": 1,
      "criteria": "FLAC lossless"
    },
    {
      "rank": 2,
      "criteria": "WAV lossless"
    },
    {
      "rank": 3,
      "criteria": "MP3/AAC ≥ 320 kbps"
    },
    {
      "rank": 4,
      "criteria": "MP3/AAC 256-319 kbps"
    },
    {
      "rank": 5,
      "criteria": "MP3/AAC < 256 kbps"
    }
  ],
  "special_cases": {
    "different_versions": {
      "description": "Album version vs Single version",
      "action": "Keep both, use comment tag to distinguish"
    },
    "remaster": {
      "description": "Original vs Remastered",
      "action": "User preference, note in comment tag"
    },
    "live_vs_studio": {
      "description": "Same song, different recording",
      "action": "NOT duplicates, keep both"
    }
  }
}
```

---

#### `release_selection_criteria`
**URI**: `music://decision/release-selection`

**Description**: How to choose between multiple MusicBrainz releases

**Content**:
```json
{
  "priority_factors": [
    {
      "rank": 1,
      "factor": "Country",
      "logic": "User region > Original release country > Worldwide",
      "reason": "Release dates and bonus tracks vary by country"
    },
    {
      "rank": 2,
      "factor": "Release Status",
      "logic": "Official > Promotion > Bootleg",
      "reason": "Official releases have verified metadata"
    },
    {
      "rank": 3,
      "factor": "Release Date",
      "logic": "Original > Remaster (unless remaster explicitly wanted)",
      "reason": "Preserves historical accuracy"
    },
    {
      "rank": 4,
      "factor": "Format",
      "logic": "Digital > CD > Vinyl",
      "reason": "Digital releases have most complete metadata"
    },
    {
      "rank": 5,
      "factor": "Tracklist Completeness",
      "logic": "More tracks > Fewer tracks (if deluxe edition)",
      "reason": "Deluxe/expanded editions include bonus content"
    }
  ],
  "decision_matrix": {
    "user_has_remaster": {
      "check": "Year in existing metadata vs MusicBrainz release year",
      "action": "Choose matching year"
    },
    "user_has_deluxe": {
      "check": "Track count in directory vs release track count",
      "action": "Choose release with matching track count"
    },
    "ambiguous": {
      "check": "Multiple releases with same attributes",
      "action": "Choose earliest release date"
    }
  },
  "examples": [
    {
      "scenario": "Album released in UK (2020) and US (2021), user in US",
      "decision": "Choose US release",
      "reason": "User region priority"
    },
    {
      "scenario": "Original (1975) vs Remastered (2011), no user preference",
      "decision": "Choose original (1975)",
      "reason": "Historical accuracy"
    },
    {
      "scenario": "Standard (12 tracks) vs Deluxe (17 tracks), user has 17 files",
      "decision": "Choose Deluxe edition",
      "reason": "Track count matches"
    }
  ]
}
```

---

### 4. Validation Resources

#### `common_mistakes`
**URI**: `music://validation/common-mistakes`

**Description**: Frequent tagging errors and how to avoid them

**Content**:
```json
{
  "mistakes": [
    {
      "category": "Album Artist",
      "mistake": "Using first track artist for compilations",
      "correct": "Album Artist = 'Various Artists'",
      "detection": "Multiple different artists in same album",
      "fix": "Update album_artist field for all tracks"
    },
    {
      "category": "Featuring Artists",
      "mistake": "Adding featured artist to main artist field",
      "example_wrong": "Artist = 'Daft Punk feat. Pharrell Williams'",
      "example_correct": "Artist = 'Daft Punk', Title = 'Get Lucky (feat. Pharrell Williams)'",
      "detection": "Artist field contains 'feat.' or 'ft.'",
      "fix": "Move featuring artist to title or separate tag"
    },
    {
      "category": "Year Field",
      "mistake": "Using release year instead of original recording year",
      "example": "Using 2023 (remaster year) instead of 1975 (original)",
      "correct": "Use original release year unless explicitly a new recording",
      "detection": "Year doesn't match MusicBrainz original release",
      "fix": "Update year to match original release"
    },
    {
      "category": "Title Case",
      "mistake": "Inconsistent capitalization",
      "example_wrong": "bohemian rhapsody",
      "example_correct": "Bohemian Rhapsody",
      "detection": "All lowercase or all uppercase titles",
      "fix": "Apply title case (capitalize major words)"
    },
    {
      "category": "Track Numbers",
      "mistake": "Including disc number in track field",
      "example_wrong": "Track = '201' (meaning disc 2, track 1)",
      "example_correct": "Track = 1, Disc = 2 (separate fields)",
      "detection": "Track number > album track count",
      "fix": "Use separate disc/track fields if supported"
    },
    {
      "category": "Genre Over-Specification",
      "mistake": "Using hyper-specific micro-genres",
      "example_wrong": "Post-Progressive Symphonic Technical Death Metal",
      "example_correct": "Metal or Death Metal",
      "detection": "Genre string > 20 characters or contains 3+ descriptors",
      "fix": "Use broad, recognized genre categories"
    }
  ],
  "validation_checklist": [
    "Album artist set for compilations?",
    "Featuring artists in title, not artist field?",
    "Year is 4-digit original release year?",
    "Title case applied consistently?",
    "Track numbers sequential without gaps?",
    "Genre is broad category, not micro-genre?"
  ]
}
```

---

#### `quality_criteria`
**URI**: `music://validation/quality-criteria`

**Description**: Metadata completeness standards

**Content**:
```json
{
  "tiers": {
    "minimal": {
      "description": "Bare minimum for usability",
      "required_fields": ["title", "artist"],
      "score": "40%",
      "use_case": "Quick identification, playback only"
    },
    "standard": {
      "description": "Good quality for most users",
      "required_fields": ["title", "artist", "album", "year", "track"],
      "recommended_fields": ["album_artist", "genre"],
      "score": "70%",
      "use_case": "Home library, media server"
    },
    "complete": {
      "description": "Comprehensive metadata",
      "required_fields": ["title", "artist", "album", "album_artist", "year", "track", "track_total", "genre"],
      "recommended_fields": ["comment", "musicbrainz_recording_id"],
      "score": "100%",
      "use_case": "Archival, professional library"
    }
  },
  "scoring_formula": {
    "title": 15,
    "artist": 15,
    "album": 15,
    "album_artist": 10,
    "year": 10,
    "track": 10,
    "track_total": 5,
    "genre": 10,
    "comment": 5,
    "musicbrainz_ids": 5
  },
  "file_quality_factors": {
    "format_score": {
      "FLAC": 100,
      "WAV": 90,
      "MP3_320": 80,
      "AAC_256": 80,
      "MP3_256": 70,
      "MP3_192": 60,
      "MP3_128": 40
    },
    "metadata_completeness": "See scoring_formula",
    "cover_art_present": 10,
    "total_max": 120
  }
}
```

---

## Implementation Notes

### For Server Developers

1. **Prompts Registration**:
   - Implement `prompts/list` handler
   - Each prompt has: name, description, arguments schema
   - Return prompt content when `prompts/get` called

2. **Resources Registration**:
   - Implement `resources/list` handler
   - Each resource has: URI, name, mimeType, description
   - Return resource content when `resources/read` called

3. **Content Format**:
   - Prompts: Markdown or plain text with structured instructions
   - Resources: JSON for structured data, Markdown for guides

4. **Dynamic Resources** (optional):
   - `library_stats`: Query actual library for real-time stats
   - `recent_operations`: Read from operation log

### For MCP Clients (LLMs)

1. **Discovery**:
   - List prompts with `prompts/list`
   - List resources with `resources/list`

2. **Usage**:
   - Use `prompts/get` to inject workflow context
   - Use `resources/read` to access reference data
   - Combine with tool calls for complete solution

3. **Best Practices**:
   - Always check resources before making assumptions
   - Use decision support resources for ambiguous cases
   - Reference educational prompts when explaining to users

---

## Maintenance

**Update Frequency**:
- Prompts: When workflows change
- Config resources: When standards evolve
- Reference resources: When format support changes
- Validation resources: When new mistakes discovered

**Versioning**:
- Include version in resource URI (e.g., `music://config/naming-schemes?v=1`)
- Deprecate old versions gracefully

**Testing**:
- Validate JSON schemas
- Test prompt workflows end-to-end
- Verify resource URIs are accessible

---

## Summary

This specification defines **9 prompts** and **9 resources** for MCP exposure:

**Prompts** (workflow guidance):
- organize_album
- clean_library
- identify_unknowns
- suggest_strategy
- compare_releases
- validate_metadata
- troubleshoot_identification
- explain_musicbrainz_structure
- metadata_best_practices

**Resources** (reference data):
- naming_schemes
- genre_taxonomy
- audio_format_guide
- metadata_field_mapping
- fingerprint_decision_tree
- duplicate_detection_rules
- release_selection_criteria
- common_mistakes
- quality_criteria

These provide rich context to LLM clients for intelligent music library management.
