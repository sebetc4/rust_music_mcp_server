# MusicBrainz Tools Enhancement Roadmap

This document outlines potential enhancements and missing features in the current MusicBrainz tools ecosystem. Each proposal includes motivation, use cases, implementation complexity, and priority.

---

## Table of Contents

1. [High Priority Enhancements](#high-priority-enhancements)
2. [Medium Priority Enhancements](#medium-priority-enhancements)
3. [Cross-Cutting Features](#cross-cutting-features)
4. [Identified Gaps](#identified-gaps)
5. [Implementation Roadmap](#implementation-roadmap)
6. [Technical Considerations](#technical-considerations)

---

## High Priority Enhancements

### 1. Relations & Enrichment Tool

**Tool Name**: `mb_get_relations`

**Motivation**: Current tools return isolated entities without relationship context. MusicBrainz's power lies in its rich relationship graph between entities.

**Parameters**:
```typescript
interface MbGetRelationsParams {
  entity_type: "artist" | "release" | "recording" | "work" | "label";
  mbid: string;
  relation_types?: string[];  // Optional filter: ["composer", "producer", "cover-art"]
  include_reverse?: boolean;   // Include relationships pointing to this entity
  limit?: number;              // Max relationships to return
}
```

**Response**:
```typescript
interface RelationsResult {
  entity_mbid: string;
  entity_type: string;
  relations: Relation[];
  total_count: number;
}

interface Relation {
  relation_type: string;        // "producer", "composer", "cover", etc.
  target_entity_type: string;   // "artist", "recording", etc.
  target_mbid: string;
  target_name: string;
  attributes?: string[];        // Relation attributes
  begin_date?: string;
  end_date?: string;
  direction: "forward" | "backward";
}
```

**Use Cases**:
- Find all producers of an album
- Identify cover versions of a song
- Discover artist collaborations
- Trace songwriting credits
- Map recording lineage (original → remix → cover)

**Implementation Complexity**: Medium (2-3 days)
- API endpoint: `/ws/2/{entity}/{mbid}?inc=*-rels`
- Response parsing can be complex
- Relationship type taxonomy is extensive

**Priority**: ⭐⭐⭐⭐⭐ (Critical for enrichment workflows)

**Example**:
```bash
# Find all collaborators on an album
{
  "entity_type": "release",
  "mbid": "18079f7b-78c3-3980-b16e-c5db63cc10a5",  // OK Computer
  "relation_types": ["producer", "engineer", "performer"]
}
```

---

### 2. External ID Lookup Tool

**Tool Name**: `mb_lookup_external`

**Motivation**: Users often have IDs from popular services (Spotify, Apple Music, Discogs) and need to map them to MusicBrainz for enrichment.

**Parameters**:
```typescript
interface MbLookupExternalParams {
  service: "spotify" | "discogs" | "bandcamp" | "youtube" | "itunes" | "amazon" | "deezer";
  external_id: string;
  entity_type?: "artist" | "release" | "recording";  // Optional hint
}
```

**Response**:
```typescript
interface ExternalLookupResult {
  service: string;
  external_id: string;
  musicbrainz_matches: EntityMatch[];
  confidence: number;  // 0-1
}

interface EntityMatch {
  entity_type: string;
  mbid: string;
  name: string;
  match_score: number;
  external_urls: Record<string, string>;
}
```

**Use Cases**:
- Import Spotify playlist metadata
- Sync library with Apple Music
- Cross-reference Discogs collection
- Enrich YouTube Music library
- Migrate from other music services

**Implementation Complexity**: Medium (1-2 days)
- MusicBrainz stores external URLs as relationships
- Query pattern: Search by URL relationship
- May require fuzzy matching for some services

**Priority**: ⭐⭐⭐⭐⭐ (Essential for service integration)

**Example**:
```bash
# Lookup Spotify track in MusicBrainz
{
  "service": "spotify",
  "external_id": "3n3Ppam7vgaVa1iaRUc9Lp",  // "Mr. Brightside"
  "entity_type": "recording"
}
```

**API Pattern**:
```
GET /ws/2/url?resource=https://open.spotify.com/track/{id}&inc=recording-rels
```

---

### 3. Batch Audio Identification

**Tool Name**: `mb_batch_identify`

**Motivation**: Current `mb_identify_record` processes one file at a time. Real-world usage involves processing entire directories/libraries.

**Parameters**:
```typescript
interface MbBatchIdentifyParams {
  file_paths: string[];
  metadata_level: "minimal" | "basic" | "full";
  max_concurrent: number;      // Respect rate limiting (default: 1)
  continue_on_error: boolean;  // Don't stop on individual failures
  timeout_per_file?: number;   // ms
}
```

**Response**:
```typescript
interface BatchIdentificationResult {
  total_files: number;
  successful: number;
  failed: number;
  results: FileIdentificationResult[];
  errors: FileError[];
  processing_time_ms: number;
}

interface FileIdentificationResult {
  file_path: string;
  status: "success" | "no_match" | "error";
  matches?: FingerprintMatch[];
  error?: string;
}
```

**Use Cases**:
- Process entire music library
- Bulk import unknown files
- Cleanup downloaded music
- Automated library organization
- Quality assurance checks

**Implementation Complexity**: High (3-4 days)
- Rate limiting coordination (max 3 requests/second to AcoustID)
- Thread pool management
- Progress tracking
- Error recovery
- Memory management for large batches

**Priority**: ⭐⭐⭐⭐ (High value for end users)

**Technical Challenges**:
- MusicBrainz rate limit: 1 req/sec
- AcoustID rate limit: 3 req/sec
- Must serialize requests or implement token bucket
- Progress reporting over MCP protocol

**Example**:
```bash
# Identify all files in a directory
{
  "file_paths": [
    "/music/unknown/track1.mp3",
    "/music/unknown/track2.mp3",
    "/music/unknown/track3.mp3"
  ],
  "metadata_level": "basic",
  "max_concurrent": 1,
  "continue_on_error": true
}
```

---

### 4. Metadata Validation Tool

**Tool Name**: `mb_validate_metadata`

**Motivation**: Detect missing, incorrect, or inconsistent metadata before writing to files. Helps maintain library quality.

**Parameters**:
```typescript
interface MbValidateMetadataParams {
  file_path: string;
  expected_mbid?: string;      // Optional: validate against known MBID
  strict_mode: boolean;        // Fail on warnings or only errors
  validation_rules?: string[]; // ["required_fields", "format_consistency", "mbid_match"]
}
```

**Response**:
```typescript
interface ValidationResult {
  file_path: string;
  quality_score: number;       // 0-100
  status: "valid" | "warnings" | "errors";
  issues: ValidationIssue[];
  suggestions: Suggestion[];
  metadata_summary: MetadataSummary;
}

interface ValidationIssue {
  severity: "error" | "warning" | "info";
  field: string;
  issue_type: "missing" | "invalid" | "inconsistent" | "low_quality";
  message: string;
  current_value?: string;
}

interface Suggestion {
  field: string;
  suggested_value: string;
  confidence: number;
  source: "musicbrainz" | "acoustid" | "inference";
}
```

**Use Cases**:
- Pre-write validation
- Library quality audit
- Detect corrupted metadata
- Find incomplete albums
- Identify encoding issues

**Implementation Complexity**: Medium (2-3 days)
- Read existing metadata (reuse read_metadata)
- Compare against MusicBrainz canonical data
- Heuristic validation rules
- Quality scoring algorithm

**Priority**: ⭐⭐⭐ (Quality of life improvement)

**Validation Rules**:
- **Required fields**: title, artist, album (for tracks)
- **Format consistency**: Year format (YYYY), track numbers (1-999)
- **MBID validation**: Check if MBID exists in MusicBrainz
- **Cross-field consistency**: Album artist matches track artists
- **Encoding quality**: Check for garbled text, encoding issues

**Example**:
```bash
{
  "file_path": "/music/album/track.mp3",
  "strict_mode": false,
  "validation_rules": ["required_fields", "mbid_match"]
}
```

---

## Medium Priority Enhancements

### 5. Artist Aggregation & Statistics

**Tool Name**: `mb_aggregate_artist`

**Motivation**: Provide comprehensive artist analytics in a single request instead of multiple API calls.

**Parameters**:
```typescript
interface MbAggregateArtistParams {
  artist_mbid: string;
  include: ("discography" | "collaborations" | "timeline" | "genres" | "statistics")[];
  years_range?: {
    start: number;
    end: number;
  };
}
```

**Response**:
```typescript
interface ArtistAggregation {
  artist_mbid: string;
  artist_name: string;

  discography?: {
    total_releases: number;
    total_recordings: number;
    by_type: Record<string, number>;  // "album": 10, "single": 5
    by_decade: Record<string, number>; // "1990s": 3, "2000s": 7
  };

  collaborations?: {
    total_collaborators: number;
    top_collaborators: Collaborator[];
    collaboration_types: Record<string, number>;
  };

  timeline?: {
    career_start: string;
    career_end?: string;
    active_periods: Period[];
    major_releases: MajorRelease[];
  };

  genres?: {
    primary_genres: string[];
    all_genres: Record<string, number>;  // genre -> frequency
  };

  statistics?: {
    total_works: number;
    average_releases_per_year: number;
    most_productive_year: number;
  };
}
```

**Use Cases**:
- Artist profile pages
- Music recommendation engines
- Collection analysis
- Research and data mining
- Artist comparison

**Implementation Complexity**: High (4-5 days)
- Multiple MusicBrainz API calls
- Data aggregation and processing
- Genre extraction from tags
- Statistical calculations

**Priority**: ⭐⭐⭐ (Nice to have for analytics)

---

### 6. Similar Entity Discovery

**Tool Name**: `mb_find_similar`

**Motivation**: Enable music discovery and recommendations based on MusicBrainz relationships and metadata.

**Parameters**:
```typescript
interface MbFindSimilarParams {
  entity_type: "artist" | "release";
  reference_mbid: string;
  similarity_type: "genre" | "style" | "era" | "collaborators" | "tags";
  limit?: number;
  min_similarity?: number;  // 0-1 threshold
}
```

**Response**:
```typescript
interface SimilarEntitiesResult {
  reference_mbid: string;
  reference_name: string;
  similarity_type: string;
  similar_entities: SimilarEntity[];
}

interface SimilarEntity {
  mbid: string;
  name: string;
  similarity_score: number;  // 0-1
  matching_factors: string[];
  metadata_snippet: Record<string, any>;
}
```

**Use Cases**:
- Music discovery
- Playlist generation
- "Similar artists" features
- Genre exploration
- Collection gap finding

**Implementation Complexity**: Very High (5-7 days)
- Complex similarity algorithms
- Tag analysis
- Graph traversal (collaborator networks)
- Performance optimization
- Caching strategy

**Priority**: ⭐⭐ (Advanced feature)

---

### 7. Timeline & Career History

**Tool Name**: `mb_get_timeline`

**Motivation**: Visualize artist or label history chronologically.

**Parameters**:
```typescript
interface MbGetTimelineParams {
  entity_type: "artist" | "label";
  mbid: string;
  start_year?: number;
  end_year?: number;
  event_types?: ("release" | "formation" | "dissolution" | "collaboration")[];
}
```

**Response**:
```typescript
interface TimelineResult {
  entity_mbid: string;
  entity_name: string;
  events: TimelineEvent[];
  periods: Period[];
}

interface TimelineEvent {
  date: string;          // ISO 8601
  event_type: string;
  title: string;
  description?: string;
  related_entities: RelatedEntity[];
}

interface Period {
  name: string;          // "Early career", "Classic period"
  start_date: string;
  end_date?: string;
  characteristics: string[];
}
```

**Use Cases**:
- Artist biography generation
- Label history
- Career analysis
- Educational content
- Interactive timelines

**Implementation Complexity**: Medium (3-4 days)

**Priority**: ⭐⭐⭐ (Good for presentation)

---

### 8. Geographic Search

**Tool Name**: `mb_search_by_location`

**Motivation**: Discover artists and labels by geographic origin.

**Parameters**:
```typescript
interface MbSearchByLocationParams {
  entity_type: "artist" | "label";
  country: string;      // ISO 3166-1 alpha-2
  city?: string;
  area?: string;
  limit?: number;
  sort_by?: "name" | "founded" | "releases";
}
```

**Response**:
```typescript
interface LocationSearchResult {
  location: {
    country: string;
    city?: string;
    area?: string;
  };
  entities: LocationEntity[];
  total_count: number;
}

interface LocationEntity {
  mbid: string;
  name: string;
  entity_type: string;
  founded_date?: string;
  area_hierarchy: string[];  // ["Seattle", "Washington", "United States"]
}
```

**Use Cases**:
- Regional music discovery
- Local artist promotion
- Cultural research
- Geographic music mapping
- Tourism applications

**Implementation Complexity**: Low-Medium (2-3 days)

**Priority**: ⭐⭐ (Niche feature)

---

## Cross-Cutting Features

These enhancements apply across multiple tools.

### 1. Caching System

**Motivation**: Reduce API calls and improve response times.

**Features**:
- Local SQLite cache
- Configurable TTL per entity type
- Smart invalidation
- Offline mode for cached data
- Cache statistics

**Implementation**:
```rust
// New module: src/core/cache/
pub struct MbCache {
    store: SqliteStore,
    ttl_config: TtlConfig,
}

// Cache key: entity_type:mbid or query_hash
impl MbCache {
    pub fn get<T>(&self, key: &str) -> Option<T>;
    pub fn set<T>(&self, key: &str, value: T, ttl: Duration);
    pub fn invalidate(&self, pattern: &str);
}
```

**Priority**: ⭐⭐⭐⭐ (Performance critical)

---

### 2. Composite Workflows

**Motivation**: Enable complex multi-step operations in a single request.

**Example Workflow**: `enrich_release`
```typescript
interface WorkflowParams {
  workflow: "enrich_release" | "identify_and_tag" | "complete_album";
  input: Record<string, any>;
  steps?: string[];  // Override default workflow
}

// Workflow: enrich_release
{
  "workflow": "enrich_release",
  "input": {
    "release_mbid": "xxx"
  }
  // Auto executes:
  // 1. mb_release_search (get_tracklist)
  // 2. mb_recording_search (for each track)
  // 3. mb_artist_search (for artists)
  // 4. mb_get_relations (producer, engineer)
  // 5. Cover art fetch
}
```

**Benefits**:
- Reduce round-trips
- Atomic operations
- Simplified client code
- Optimized API usage

**Implementation Complexity**: High (4-5 days)

**Priority**: ⭐⭐⭐ (Developer experience)

---

### 3. Standardized Export Formats

**Motivation**: Enable interoperability with other music tools.

**Formats**:
- **Picard JSON**: For MusicBrainz Picard import
- **Beets YAML**: For beets library
- **ID3v2.4 mapping**: Direct tag structure
- **M3U/M3U8**: Playlist format with metadata
- **CSV**: Spreadsheet analysis

**Implementation**:
```rust
pub trait MetadataExporter {
    fn export_picard(&self, data: &MbEntity) -> String;
    fn export_beets(&self, data: &MbEntity) -> String;
    fn export_id3(&self, data: &MbEntity) -> Id3Frame;
}
```

**Priority**: ⭐⭐⭐ (Ecosystem integration)

---

## Identified Gaps

### Gap 1: No Cover Art Support

**Current State**: Tools don't fetch album artwork.

**Proposed Solution**: `mb_get_cover_art`

**Parameters**:
```typescript
interface MbGetCoverArtParams {
  release_mbid: string;
  size: "small" | "medium" | "large";  // 250px, 500px, 1200px
  format?: "jpg" | "png";
  fallback_to_release_group?: boolean;
}
```

**API**: CoverArtArchive.org API
```
GET https://coverartarchive.org/release/{mbid}/front
```

**Priority**: ⭐⭐⭐⭐ (Visual content is important)

---

### Gap 2: No Advanced Fuzzy Search

**Current State**: Exact string matching only.

**Proposed Enhancement**: Add fuzzy search parameters to existing tools

**Parameters**:
```typescript
interface FuzzySearchParams extends BaseSearchParams {
  fuzzy_threshold?: number;     // 0-1 (default: 0.8)
  phonetic_matching?: boolean;  // Use Soundex/Metaphone
  typo_tolerance?: number;      // Max Levenshtein distance
}
```

**Implementation**:
- Client-side fuzzy matching on results
- Or use Lucene query syntax: `~0.8` suffix

**Priority**: ⭐⭐⭐ (User experience)

---

### Gap 3: No Alias Support

**Current State**: Tools don't return artist aliases/alternate names.

**Proposed Enhancement**: Include aliases in all artist results

**Example**:
```typescript
interface ArtistInfo {
  name: string;
  aliases: Alias[];      // ← NEW
  sort_name: string;     // ← NEW
  // ... existing fields
}

interface Alias {
  name: string;
  sort_name: string;
  locale?: string;
  type?: "Artist name" | "Legal name" | "Search hint";
  primary: boolean;
}
```

**Implementation**: Use `?inc=aliases` in API calls

**Priority**: ⭐⭐⭐ (Localization & search)

---

### Gap 4: No Genre/Tag Aggregation

**Current State**: Genres scattered across entities.

**Proposed Enhancement**: Add genre extraction

**Example**:
```typescript
interface GenreInfo {
  name: string;
  count: number;        // How many times tagged
  vote_count: number;   // Number of user votes
  percentage: number;   // Relative frequency
}
```

**Priority**: ⭐⭐⭐ (Discovery & classification)

---

## Implementation Roadmap

### Phase 1: High-Value Quick Wins (2-3 weeks)

1. **mb_get_relations** (3 days)
   - Massive value for enrichment
   - Well-defined MusicBrainz API

2. **mb_lookup_external** (2 days)
   - Essential for service integration
   - Straightforward implementation

3. **Cover art support** (2 days)
   - Simple API integration
   - High user visibility

4. **Add aliases to artist tools** (1 day)
   - Small code change
   - Improves search quality

### Phase 2: Batch Operations (2-3 weeks)

5. **mb_batch_identify** (4-5 days)
   - Complex but high user value
   - Requires careful rate limiting

6. **Caching system** (4-5 days)
   - Infrastructure for all tools
   - Performance multiplier

### Phase 3: Advanced Features (4-6 weeks)

7. **mb_validate_metadata** (3-4 days)
8. **mb_aggregate_artist** (4-5 days)
9. **Composite workflows** (5-6 days)
10. **Export format support** (3-4 days)

### Phase 4: Discovery & Analytics (4-6 weeks)

11. **mb_find_similar** (6-7 days)
12. **mb_get_timeline** (3-4 days)
13. **mb_search_by_location** (2-3 days)
14. **Genre/tag aggregation** (2-3 days)

---

## Technical Considerations

### Rate Limiting Strategy

**MusicBrainz**: 1 request/second
**AcoustID**: 3 requests/second (with API key)

**Implementation**:
```rust
pub struct RateLimiter {
    tokens: Arc<Mutex<TokenBucket>>,
    service: ApiService,
}

impl RateLimiter {
    pub async fn acquire(&self) -> RateLimitPermit;
}
```

**Apply to all tools**:
- Shared rate limiter instance
- Token bucket algorithm
- Per-service configuration

---

### Error Handling

**Patterns**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum MbToolError {
    #[error("Rate limit exceeded, retry after {retry_after}s")]
    RateLimitExceeded { retry_after: u64 },

    #[error("External service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Invalid MBID format: {mbid}")]
    InvalidMbid { mbid: String },

    #[error("No results found for query: {query}")]
    NoResults { query: String },
}
```

---

### Testing Strategy

**Unit Tests**: Business logic
**Integration Tests**: API calls (ignored by default)
**Mock Tests**: Use recorded responses

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_relation_parsing() { ... }

    #[ignore]
    #[test]
    fn test_api_get_relations() { ... }

    #[test]
    fn test_with_mock_response() {
        let mock = load_fixture("relations.json");
        // ...
    }
}
```

---

### Performance Targets

| Operation | Target Latency | Notes |
|-----------|---------------|-------|
| Search (cached) | < 50ms | Local cache hit |
| Search (API) | < 500ms | MusicBrainz API |
| Identify (fingerprint) | < 2s | Chromaprint + AcoustID |
| Batch identify (10 files) | < 15s | With rate limiting |
| Relations fetch | < 300ms | Single API call |

---

## Migration Path for Existing Clients

### Backward Compatibility

All new tools are additive - no breaking changes to existing tools.

### Deprecation Policy

If enhancing existing tools with new parameters:
1. Add new parameters as optional
2. Maintain old behavior as default
3. Document new features
4. Provide migration examples

### Example Migration

```typescript
// Old way
const artist = await mb_artist_search({ query: "Radiohead" });
const releases = await mb_artist_search({
  search_type: "artist_releases",
  query: artist.mbid
});

// New way (with aggregation)
const artistData = await mb_aggregate_artist({
  artist_mbid: artist.mbid,
  include: ["discography", "collaborations"]
});
// Single request, richer data
```

---

## Conclusion

This roadmap provides a clear path to significantly enhance the MusicBrainz tools ecosystem. Implementation should prioritize:

1. **Quick wins** (Phase 1) for immediate value
2. **Infrastructure** (caching, rate limiting) for scalability
3. **Advanced features** for power users
4. **Maintain quality** (tests, docs, idiomatic Rust)

**Estimated Total Effort**: 12-16 weeks (3-4 months) for all phases.

**Recommended Starting Point**: Phase 1 - High-value quick wins.

---

## See Also

- [../tools/mb/README.md](../tools/mb/README.md) - Current MusicBrainz tools
- [../../CLAUDE.md](../../CLAUDE.md) - Development guidelines
- [../guides/adding-tools.md](../guides/adding-tools.md) - Tool implementation guide
- [MusicBrainz API Documentation](https://musicbrainz.org/doc/MusicBrainz_API)
- [AcoustID API Documentation](https://acoustid.org/webservice)
