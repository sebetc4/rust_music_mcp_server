# 🎵 Music MCP Server

A Rust-based [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server for automated music library organization.

Identify audio files, enrich metadata, and organize your music collection using MusicBrainz and AcoustID.

---

## ✨ Features

- **🎯 Audio Fingerprinting**: Identify tracks using Chromaprint/AcoustID
- **📝 Metadata Management**: Read and write audio tags (MP3, FLAC, M4A, WAV, OGG)
- **🔍 MusicBrainz Integration**: Search artists, releases, and recordings
- **📁 Safe File Operations**: List and rename files with dry-run support
- **🚀 Three Transport Options**: STDIO (CLI), TCP (network), HTTP (REST API)
- **⚡ High Performance**: Written in Rust with async/await
- **🛡️ Type Safe**: Strong typing throughout with compile-time validation

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      MCP Clients                         │
│        (AI Agents, CLI Tools, Web Apps, etc.)            │
└─────────────────────────────────────────────────────────┘
                            │
         ┌──────────────────┼──────────────────┐
         │                  │                  │
    ┌────▼────┐       ┌─────▼─────┐     ┌────▼─────┐
    │  STDIO  │       │    TCP    │     │   HTTP   │
    │(default)│       │  (socket) │     │  (Axum)  │
    └────┬────┘       └─────┬─────┘     └────┬─────┘
         │                  │                 │
         └──────────────────┼─────────────────┘
                            │
              ┌─────────────▼─────────────┐
              │    MCP Server Core         │
              │    (rmcp 0.11.0)           │
              └─────────────┬──────────────┘
                            │
              ┌─────────────▼─────────────┐
              │      Domain Layer          │
              │                            │
              │  ┌──────────────────────┐  │
              │  │  Tools (9 total)     │  │
              │  │  • Filesystem (2)    │  │
              │  │  • Metadata (2)      │  │
              │  │  • MusicBrainz (5)   │  │
              │  └──────────────────────┘  │
              │                            │
              │  ┌──────────────────────┐  │
              │  │  Resources & Prompts │  │
              │  └──────────────────────┘  │
              └─────────────┬──────────────┘
                            │
              ┌─────────────▼─────────────┐
              │    External Services       │
              │  • MusicBrainz API         │
              │  • AcoustID API            │
              │  • Chromaprint (fpcalc)    │
              └────────────────────────────┘
```

---

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.70+ ([Install Rust](https://rustup.rs/))
- **Chromaprint**: For audio fingerprinting

```bash
# Ubuntu/Debian
sudo apt-get install libchromaprint-tools

# macOS
brew install chromaprint

# Windows
# Download from https://acoustid.org/chromaprint
```

### Installation

```bash
# Clone repository
git clone <repository-url>
cd music_mcp_server

# Configure environment (optional but recommended)
cp .env.example .env
# Edit .env and set MCP_ACOUSTID_API_KEY (get free key at https://acoustid.org/api-key)

# Build (STDIO only, fastest)
cargo build --release

# Or build with all transport options
cargo build --release --features all
```

### Configuration

The server is configured via environment variables. See [Configuration Guide](documentation/guides/configuration.md) for details.

**Quick setup**:
```bash
# Copy example configuration
cp .env.example .env

# Get your AcoustID API key (free)
# Visit: https://acoustid.org/api-key

# Add to .env:
echo "MCP_ACOUSTID_API_KEY=your_key_here" >> .env
```

**Key environment variables**:
- `MCP_TRANSPORT`: Choose `stdio` (default), `tcp`, or `http`
- `MCP_ACOUSTID_API_KEY`: Your AcoustID API key for audio identification
- `MCP_LOG_LEVEL`: Log level (`info`, `debug`, `trace`)

See [`.env.example`](.env.example) for all available options.

### Running the Server

#### STDIO (Default - for CLI tools)

```bash
cargo run --release
```

#### TCP (for network clients)

```bash
cargo run --release --features tcp
```

#### HTTP (for web applications)

```bash
cargo run --release --features http

# Server starts at http://localhost:8080
```

---

## 🛠️ Available Tools

| Tool | Description | Category |
|------|-------------|----------|
| **fs_list_dir** | List directory contents with optional details | Filesystem |
| **fs_rename** | Rename files with dry-run support | Filesystem |
| **read_metadata** | Read audio tags from music files | Metadata |
| **write_metadata** | Write/update audio tags | Metadata |
| **mb_artist_search** | Search artists and get their releases | MusicBrainz |
| **mb_release_search** | Search releases and get tracklists | MusicBrainz |
| **mb_recording_search** | Search recordings and find where they appear | MusicBrainz |
| **mb_advanced_search** | Lucene-style queries across all entities | MusicBrainz |
| **mb_identify_record** | Identify audio files via fingerprinting | MusicBrainz |

---

## 📖 Example Usage

### Identify an Unknown Audio File

```bash
# Using HTTP transport
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "mb_identify_record",
      "arguments": {
        "file_path": "/music/unknown_track.mp3",
        "metadata_level": "full"
      }
    }
  }'
```

### Search for an Artist

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "mb_artist_search",
      "arguments": {
        "artist": "Radiohead",
        "include_releases": true
      }
    }
  }'
```

### Read Audio Metadata

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "read_metadata",
      "arguments": {
        "file_path": "/music/song.mp3"
      }
    }
  }'
```

---

## 🏗️ Technology Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| **Language** | Rust | 1.70+ |
| **MCP SDK** | rmcp | 0.11.0 |
| **Async Runtime** | Tokio | 1.48 |
| **HTTP Server** | Axum | 0.8 |
| **Audio Metadata** | Lofty | 0.22.4 |
| **MusicBrainz** | musicbrainz_rs | 0.12 |
| **Fingerprinting** | rusty-chromaprint | 0.3.0 |
| **Audio Decoding** | Symphonia | 0.5 |

---

## 📚 Documentation

Comprehensive documentation is available in the [`documentation/`](documentation/README.md) directory:

### Getting Started
- [Configuration Guide](documentation/guides/configuration.md) - Environment variables and setup
- [Adding New Tools](documentation/guides/adding-tools.md) - Step-by-step tutorial

### Architecture
- [System Overview](documentation/architecture/overview.md) - High-level architecture
- [Transport Layer](documentation/architecture/transport-layer.md) - STDIO/TCP/HTTP details
- [Tool System](documentation/architecture/tool-system.md) - Dual handler pattern

### Tools Reference
- [Filesystem Tools](documentation/tools/filesystem-tools.md) - File operations
- [Metadata Tools](documentation/tools/metadata-tools.md) - Audio tag management
- [MusicBrainz Tools](documentation/tools/musicbrainz-tools.md) - Complete MB tools guide

### For Developers
- [AI Agent Guidelines](CLAUDE.md) - Rules for AI-assisted development
- [Testing Guide](documentation/guides/testing.md) - Testing strategies
- [Error Handling](documentation/reference/error-handling.md) - Error patterns

---

## 🧪 Testing

### Run All Tests

```bash
cargo test --features all
```

### Run Network Tests (Rate-Limited)

```bash
cargo test -- --ignored --test-threads=1
```

### Test Specific Transport

```bash
# Test HTTP transport
python scripts/test_http_client.py

# Test TCP transport
python scripts/test_tcp_client.py

# Test STDIO transport
python scripts/test_stdio_client.py
```

---

## 🔧 Configuration

Configure via environment variables:

### Server Configuration

```bash
export MCP_SERVER_NAME="My Music Server"
export MCP_LOG_LEVEL="info"  # debug, info, warn, error
```

### HTTP Transport

```bash
export MCP_HTTP_HOST="0.0.0.0"        # Bind address
export MCP_HTTP_PORT="8080"           # Port
export MCP_HTTP_CORS_ENABLED="true"   # Enable CORS
```

### TCP Transport

```bash
export MCP_TCP_HOST="127.0.0.1"       # Bind address
export MCP_TCP_PORT="3000"            # Port
```

See [Configuration Guide](documentation/guides/configuration.md) for complete details.

---

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

### For AI Agents

Read [CLAUDE.md](CLAUDE.md) for development rules and best practices.

### For Humans

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Follow Rust idioms**: Use `cargo fmt` and `cargo clippy`
4. **Write tests**: Ensure `cargo test --features all` passes
5. **Document changes**: Update relevant documentation
6. **Commit changes**: `git commit -m 'Add amazing feature'`
7. **Push to branch**: `git push origin feature/amazing-feature`
8. **Open Pull Request**

### Code Standards

- ✅ All code in English
- ✅ No `unwrap()` in production code
- ✅ Strong typing with domain concepts
- ✅ Tests for new features
- ✅ Documentation for public APIs

---

## 🐛 Troubleshooting

### "fpcalc not found"

**Solution**: Install Chromaprint

```bash
# Ubuntu/Debian
sudo apt-get install libchromaprint-tools

# macOS
brew install chromaprint
```

### "Address already in use"

**Solution**: Change port or kill existing process

```bash
# Find process using port
lsof -i :8080

# Use different port
MCP_HTTP_PORT=9000 cargo run --features http
```

### "Rate limit exceeded"

**Solution**: MusicBrainz API is rate-limited to 1 request/second

```bash
# Run tests sequentially
cargo test -- --ignored --test-threads=1
```

See [Troubleshooting Guide](documentation/reference/troubleshooting.md) for more.

---

## 📊 Project Structure

```
music_mcp_server/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── core/                      # Core infrastructure
│   │   ├── config.rs              # Configuration management
│   │   ├── error.rs               # Error types
│   │   ├── server.rs              # MCP server implementation
│   │   └── transport/             # Transport implementations
│   │       ├── stdio.rs           # STDIO transport (default)
│   │       ├── tcp.rs             # TCP transport
│   │       └── http.rs            # HTTP transport
│   ├── domains/                   # Domain logic
│   │   ├── tools/                 # MCP tools
│   │   │   ├── definitions/       # Tool implementations
│   │   │   │   ├── fs/            # Filesystem tools
│   │   │   │   ├── metadata/      # Metadata tools
│   │   │   │   └── mb/            # MusicBrainz tools
│   │   │   ├── registry.rs        # HTTP tool registry
│   │   │   └── router.rs          # STDIO/TCP tool router
│   │   ├── resources/             # MCP resources
│   │   └── prompts/               # MCP prompts
│   └── tests/                     # Integration tests
├── scripts/                       # Test scripts
├── documentation/                 # Comprehensive docs
├── examples/                      # Usage examples
├── Cargo.toml                     # Dependencies & features
├── CLAUDE.md                      # AI development guidelines
└── README.md                      # This file
```

---

## 🔗 Links

- **MCP Protocol**: https://modelcontextprotocol.io/
- **MusicBrainz**: https://musicbrainz.org/
- **AcoustID**: https://acoustid.org/
- **Chromaprint**: https://acoustid.org/chromaprint
- **rmcp SDK**: https://docs.rs/rmcp/

---

## 📄 License

[License information to be added]

---

## 🙏 Acknowledgments

- **MusicBrainz** community for the incredible music database
- **AcoustID** for audio fingerprinting technology
- **rmcp** developers for the excellent MCP SDK
- **Rust community** for the amazing ecosystem

---

**Made with ❤️ and Rust**
