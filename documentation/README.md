# Music MCP Server - Technical Documentation

Welcome to the comprehensive technical documentation for the Music MCP Server project.

> **Quick Reference**: See [../CLAUDE.md](../CLAUDE.md) for AI agent development guidelines.

---

## Quick Links

### Getting Started
- [Configuration Guide](guides/configuration.md) - Environment variables, feature flags, and setup
- [Adding New Tools](guides/adding-tools.md) - Step-by-step tutorial for implementing new MCP tools

### Architecture
- [System Overview](architecture/overview.md) - High-level 3-layer architecture
- [Configuration Workflow](architecture/config-workflow.md) - Environment variables and config propagation
- [Transport Layer](architecture/transport-layer.md) - STDIO, TCP, and HTTP implementations
- [Tool System](architecture/tool-system.md) - Dual handler pattern and registration flow
- [External APIs](architecture/external-apis.md) - MusicBrainz and AcoustID integration details

### Tools Reference
- [Filesystem Tools](tools/filesystem-tools.md) - `fs_list_dir`, `fs_rename`
- [Metadata Tools](tools/metadata-tools.md) - `read_metadata`, `write_metadata`
- [MusicBrainz Tools](tools/musicbrainz-tools.md) - All 5 MB tools with examples

### Deep Dives
- [Error Handling](reference/error-handling.md) - Error types, patterns, and best practices
- [Dependencies](reference/dependencies.md) - Key crates and their usage
- [Music Domain](reference/music-domain.md) - Music concepts and metadata normalization
- [Testing Strategy](guides/testing.md) - Unit tests, integration tests, Python scripts

---

## Documentation Structure

```
documentation/
├── README.md (this file)          # Documentation hub
│
├── architecture/                   # Technical implementation details
│   ├── overview.md                # 3-layer architecture
│   ├── transport-layer.md         # STDIO/TCP/HTTP details
│   ├── tool-system.md             # Tool pattern & dual handler
│   └── external-apis.md           # MusicBrainz + AcoustID
│
├── guides/                         # Step-by-step tutorials
│   ├── adding-tools.md            # How to add new MCP tools
│   ├── configuration.md           # Environment & feature flags
│   └── testing.md                 # Testing approach
│
├── tools/                          # Individual tool documentation
│   ├── filesystem-tools.md        # File operations (2 tools)
│   ├── metadata-tools.md          # Audio metadata (2 tools)
│   └── musicbrainz-tools.md       # MusicBrainz integration (5 tools)
│
└── reference/                      # In-depth technical topics
    ├── error-handling.md          # Error types & patterns
    ├── dependencies.md            # Crate documentation
    └── music-domain.md            # Domain concepts
```

---

## Project Overview

The Music MCP Server is a **Rust-based MCP (Model Context Protocol) server** designed for automated music library organization. It provides:

- **Audio fingerprinting** via Chromaprint/AcoustID
- **Metadata enrichment** from MusicBrainz
- **Safe file operations** with dry-run support
- **Multiple transport layers** (STDIO, TCP, HTTP)

### Technology Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| MCP SDK | rmcp | 0.11.0 |
| Metadata | lofty | 0.22.4 |
| MusicBrainz | musicbrainz_rs | 0.12 |
| Fingerprinting | rusty-chromaprint | 0.3.0 |
| HTTP Server | axum | 0.8 |
| Async Runtime | tokio | 1.48 |

### Architecture at a Glance

```
┌─────────────────────────────────────────────────────────────┐
│                        MCP Client                            │
│                   (AI Agent, CLI, etc.)                      │
└─────────────────────────────────────────────────────────────┘
                             │
                    ┌────────┴────────┐
                    │                 │
            ┌───────▼───────┐  ┌──────▼──────┐  ┌──────────┐
            │  STDIO         │  │    TCP      │  │   HTTP   │
            │  (default)     │  │  (port)     │  │  (Axum)  │
            └───────┬────────┘  └──────┬──────┘  └─────┬────┘
                    │                  │                │
                    └──────────────────┴────────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │      MCP Server Core         │
                    │  (rmcp 0.11.0 protocol)      │
                    └──────────────┬──────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │       Domain Layer           │
                    │  ┌────────────────────────┐  │
                    │  │  Tools (9 total)       │  │
                    │  │  - Filesystem (2)      │  │
                    │  │  - Metadata (2)        │  │
                    │  │  - MusicBrainz (5)     │  │
                    │  └────────────────────────┘  │
                    │  ┌────────────────────────┐  │
                    │  │  Resources & Prompts   │  │
                    │  └────────────────────────┘  │
                    └──────────────┬──────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │    External Services         │
                    │  - MusicBrainz API           │
                    │  - AcoustID API              │
                    │  - Chromaprint (fpcalc)      │
                    └──────────────────────────────┘
```

---

## Available Tools (9 Total)

| Tool Name | Category | Description |
|-----------|----------|-------------|
| `fs_list_dir` | Filesystem | List directory contents with optional details |
| `fs_rename` | Filesystem | Rename files with dry-run support |
| `read_metadata` | Metadata | Read audio tags (MP3, FLAC, M4A, WAV, OGG) |
| `write_metadata` | Metadata | Write/update audio tags |
| `mb_artist_search` | MusicBrainz | Search artists, get releases |
| `mb_release_search` | MusicBrainz | Search releases, get tracklists |
| `mb_recording_search` | MusicBrainz | Search recordings |
| `mb_advanced_search` | MusicBrainz | Lucene-style queries across entities |
| `mb_identify_record` | MusicBrainz | Audio fingerprinting via AcoustID |

---

## Contributing to Documentation

When adding new documentation:

1. **Choose the right category**:
   - `architecture/` - Implementation details, patterns
   - `guides/` - How-to tutorials
   - `tools/` - Tool-specific docs
   - `reference/` - Deep technical topics

2. **Use ASCII diagrams** for visual representations

3. **Include code examples** from actual implementation

4. **Cross-reference** related documents

5. **Keep English-only** (project requirement)

---

## Next Steps

- **New to the project?** Start with [System Overview](architecture/overview.md)
- **Adding a tool?** See [Adding New Tools](guides/adding-tools.md)
- **Understanding transports?** Read [Transport Layer](architecture/transport-layer.md)
- **Working with MusicBrainz?** Check [MusicBrainz Tools](tools/musicbrainz-tools.md)

For AI agent development guidelines, see [../CLAUDE.md](../CLAUDE.md).
