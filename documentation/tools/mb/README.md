# MusicBrainz Tools Documentation

Individual documentation files for each MusicBrainz tool.

---

## Structure

This directory contains dedicated documentation for each MusicBrainz tool, mirroring the code structure in `src/domains/tools/definitions/mb/`.

### Tool Documentation

| File | Corresponding Code | Description |
|------|-------------------|-------------|
| [mb_artist_search.md](mb_artist_search.md) | `artist.rs` | Artist search and discography |
| [mb_release_search.md](mb_release_search.md) | `release.rs` | Release search and tracklists |
| [mb_recording_search.md](mb_recording_search.md) | `recording.rs` | Recording search and appearances |
| [mb_advanced_search.md](mb_advanced_search.md) | `advanced.rs` | Advanced Lucene queries |
| [mb_identify_record.md](mb_identify_record.md) | `identify_record.rs` | Audio fingerprinting |

### Shared Documentation

| File | Description |
|------|-------------|
| [common-concepts.md](common-concepts.md) | Shared concepts (MBIDs, entity types, etc.) |
| [rate-limiting.md](rate-limiting.md) | API rate limits and best practices |
| [troubleshooting.md](troubleshooting.md) | Common issues and solutions |

---

## Navigation

### Start Here
- **New to MusicBrainz tools?** → Start with [../musicbrainz-tools.md](../musicbrainz-tools.md)
- **Looking for a specific tool?** → See tool files above
- **Having issues?** → Check [troubleshooting.md](troubleshooting.md)

### Quick Links

**By task**:
- Find an artist → [mb_artist_search.md](mb_artist_search.md)
- Find an album → [mb_release_search.md](mb_release_search.md)
- Find a track → [mb_recording_search.md](mb_recording_search.md)
- Complex query → [mb_advanced_search.md](mb_advanced_search.md)
- Identify audio → [mb_identify_record.md](mb_identify_record.md)

**By topic**:
- Understanding MBIDs → [common-concepts.md](common-concepts.md#musicbrainz-identifiers-mbids)
- Rate limits → [rate-limiting.md](rate-limiting.md)
- No results found → [troubleshooting.md](troubleshooting.md#no-results-found)

---

## Maintenance

### When to Update

Update documentation when:
1. Tool parameters change
2. New features added
3. API behavior changes
4. Common issues discovered
5. Examples become outdated

### Update Checklist

When modifying a tool (e.g., `artist.rs`):
- [ ] Update corresponding doc ([mb_artist_search.md](mb_artist_search.md))
- [ ] Update examples if API changed
- [ ] Update shared docs if concepts changed
- [ ] Update index ([../musicbrainz-tools.md](../musicbrainz-tools.md)) if tool added/removed
- [ ] Test all example requests still work

### File Naming Convention

- Tool docs: `mb_{tool_name}.md` (matches MCP tool name)
- Shared docs: `{topic}.md` (kebab-case)
- All lowercase, hyphen-separated

---

## Architecture Benefits

### Why Individual Files?

**Easier maintenance**:
- Update one tool's docs without affecting others
- Parallel work on different tools
- Clear responsibility (one tool = one file)

**Better navigation**:
- Direct links to specific tools
- Smaller, focused documents
- Quick reference

**Code alignment**:
```
src/domains/tools/definitions/mb/artist.rs
→ documentation/tools/mb/mb_artist_search.md

Changes in code → Update matching doc file
```

### Code-to-Doc Mapping

| Code File | Documentation File | Purpose |
|-----------|-------------------|---------|
| `artist.rs` | `mb_artist_search.md` | Artist search implementation & docs |
| `release.rs` | `mb_release_search.md` | Release search implementation & docs |
| `recording.rs` | `mb_recording_search.md` | Recording search implementation & docs |
| `advanced.rs` | `mb_advanced_search.md` | Advanced search implementation & docs |
| `identify_record.rs` | `mb_identify_record.md` | Identification implementation & docs |
| `common.rs` | `common-concepts.md` | Shared utilities & concepts |

---

## Contributing

### Adding a New Tool

1. **Create tool file**: `src/domains/tools/definitions/mb/new_tool.rs`
2. **Create doc file**: `documentation/tools/mb/mb_new_tool.md`
3. **Update index**: Add to [../musicbrainz-tools.md](../musicbrainz-tools.md)
4. **Update this README**: Add to tables above

### Documentation Template

Each tool doc should include:
- Overview
- Parameters (with TypeScript signatures)
- Examples (3-5 practical examples)
- Use cases
- Response fields
- Related tools
- Common patterns
- Links to shared docs

See existing files for template structure.

---

## See Also

- [../../CLAUDE.md](../../CLAUDE.md) - AI agent development guide
- [../README.md](../README.md) - Documentation overview
- [../../architecture/tool-system.md](../../architecture/tool-system.md) - Tool architecture
