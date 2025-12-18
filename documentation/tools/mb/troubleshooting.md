# Troubleshooting

Common issues and solutions for MusicBrainz tools.

---

## Table of Contents

1. [Search Issues](#search-issues)
2. [Identification Issues](#identification-issues)
3. [Network Issues](#network-issues)
4. [Rate Limiting Issues](#rate-limiting-issues)
5. [Data Quality Issues](#data-quality-issues)
6. [System Issues](#system-issues)

---

## Search Issues

### "No results found"

**Problem**: Search returns zero results.

**Possible Causes**:
1. Typo in search term
2. Artist/release not in MusicBrainz database
3. Search too specific
4. Incorrect spelling or alternate name

**Solutions**:

**1. Check spelling**:
```json
// Typo
{"artist": "Radiohad"}  // ❌

// Correct
{"artist": "Radiohead"}  // ✅
```

**2. Try broader search terms**:
```json
// Too specific
{"release": "OK Computer (Collector's Edition)"}  // ❌

// Broader
{"release": "OK Computer"}  // ✅
```

**3. Use wildcards (advanced search)**:
```json
{
  "entity": "artist",
  "query": "artist:radio*"  // Matches Radiohead, Radio Dept., etc.
}
```

**4. Try alternate names**:
```json
// Some artists use "The", some don't
{"artist": "The Beatles"}  // Try first
{"artist": "Beatles"}      // Try second
```

**5. Check if entity exists**:
Visit https://musicbrainz.org/ and search manually.

---

### Too Many Results (Ambiguous)

**Problem**: Search returns hundreds of unrelated results.

**Possible Causes**:
1. Common/generic search term
2. Missing artist filter
3. Need more specific criteria

**Solutions**:

**1. Add artist filter**:
```json
// Ambiguous
{"recording": "One"}  // ❌ Hundreds of results

// Specific
{"recording": "One", "artist": "U2"}  // ✅
```

**2. Use advanced search with multiple criteria**:
```json
{
  "entity": "release",
  "query": "release:Discovery AND artist:\"Daft Punk\" AND date:2001"
}
```

**3. Reduce limit**:
```json
{"artist": "Smith", "limit": 5}  // Get top 5 only
```

**4. Use exact phrase matching**:
```json
{
  "entity": "artist",
  "query": "artist:\"Daft Punk\""  // Exact match
}
```

---

### "MBID not found"

**Problem**: Direct MBID lookup fails.

**Possible Causes**:
1. Invalid MBID format
2. MBID was deleted from MusicBrainz
3. MBID was merged with another entity
4. Typo in MBID

**Solutions**:

**1. Verify MBID format**:
```
Valid:   a74b1b7f-71a5-4011-9441-d0b5e4122711
Invalid: a74b1b7f-71a5-4011-9441-d0b5e412271   (too short)
Invalid: a74b1b7f-71a5-4011-9441-d0b5e4122g11  (invalid char 'g')
```

**2. Check MBID on website**:
Visit: `https://musicbrainz.org/artist/{mbid}`

**3. Search by name instead**:
```json
// MBID lookup failed, try name
{"artist": "Radiohead"}
```

**4. Check for merges**:
Entity may have been merged; MusicBrainz website will redirect to new MBID.

---

## Identification Issues

### "fpcalc not found"

**Problem**: `mb_identify_record` fails with fpcalc error.

**Error Message**:
```
Error: fpcalc binary not found

The fpcalc tool is required for audio fingerprinting.
```

**Cause**: Chromaprint package not installed.

**Solutions**:

**Ubuntu/Debian**:
```bash
sudo apt-get update
sudo apt-get install libchromaprint-tools
```

**macOS**:
```bash
brew install chromaprint
```

**Windows**:
1. Download from: https://acoustid.org/chromaprint
2. Extract to `C:\Program Files\Chromaprint`
3. Add to PATH: `C:\Program Files\Chromaprint`

**Verify installation**:
```bash
fpcalc -version
# Should output: fpcalc version X.X.X
```

**Still not working?**
- Check if fpcalc is in PATH: `which fpcalc` (Linux/Mac) or `where fpcalc` (Windows)
- Try absolute path in configuration (if supported)

---

### "File not found"

**Problem**: Identification fails because audio file not found.

**Error Message**:
```
Error: File not found: /music/track.mp3
```

**Possible Causes**:
1. Incorrect file path
2. File doesn't exist
3. No read permissions
4. Relative vs absolute path issue

**Solutions**:

**1. Verify file exists**:
```bash
ls -l /music/track.mp3
```

**2. Use absolute path**:
```json
// Relative (may fail)
{"file_path": "music/track.mp3"}  // ❌

// Absolute (more reliable)
{"file_path": "/home/user/music/track.mp3"}  // ✅
```

**3. Check permissions**:
```bash
# Check current permissions
ls -l /music/track.mp3

# Fix if needed
chmod +r /music/track.mp3
```

**4. Escape special characters**:
```bash
# File with spaces
/music/My Music/track 01.mp3  # ❌ May fail

"/music/My Music/track 01.mp3"  # ✅ Quoted
```

---

### "No matches found" (Identification)

**Problem**: Fingerprint generated but no matches in AcoustID.

**Response**:
```
Audio Identification Results
============================

File: /music/track.mp3
Fingerprint generated successfully (duration: 180.2s)

No matches found in AcoustID database.
```

**Possible Causes**:
1. Recording not in MusicBrainz database
2. Poor audio quality
3. Heavily edited or remixed audio
4. Rare/unreleased recording
5. Non-music audio (podcast, audiobook, etc.)

**Solutions**:

**1. Try different quality file**:
If you have multiple versions (FLAC, high-bitrate MP3), try those.

**2. Check audio quality**:
```bash
# Check file properties
ffprobe /music/track.mp3

# Low bitrate (< 128kbps) may cause issues
```

**3. Try manual search**:
```json
// If you know track name/artist
{
  "name": "mb_recording_search",
  "arguments": {
    "recording": "Track Name",
    "artist": "Artist Name"
  }
}
```

**4. Check if it's music**:
AcoustID only works for music, not:
- Podcasts
- Audiobooks
- Sound effects
- Spoken word

**5. Consider adding to MusicBrainz**:
If it's legitimate music not in the database, consider contributing:
https://musicbrainz.org/doc/How_to_Add_a_Release

---

### Low Confidence Matches

**Problem**: Matches returned but with low scores (< 0.70).

**Response**:
```
Match #1 (Score: 0.62)
Recording: Possible Match
```

**Causes**:
1. Audio quality issues
2. Different mix/version
3. Speed/pitch alterations
4. Heavy editing

**Solutions**:

**1. Manually verify**:
Compare metadata against known information:
- Duration matches?
- Artist correct?
- Album title reasonable?

**2. Try original source**:
If file is a transcode/rip, try original CD/FLAC.

**3. Check for edits**:
- Fade in/out added?
- Speed changed?
- Pitch shifted?

**4. Use higher-quality source**:
Better audio = better fingerprint = better matches.

**5. Manual search as fallback**:
Use `mb_recording_search` with known details.

---

### "Unsupported format"

**Problem**: fpcalc can't process audio file.

**Error Message**:
```
Error: Failed to generate fingerprint for /music/file.xyz
```

**Causes**:
1. Unsupported audio format
2. Corrupted file
3. Not an audio file

**Solutions**:

**1. Check format**:
```bash
file /music/file.xyz
```

**2. Convert to supported format**:
```bash
# Convert to MP3
ffmpeg -i input.xyz -q:a 0 output.mp3

# Convert to FLAC
ffmpeg -i input.xyz output.flac
```

**3. Test file integrity**:
```bash
# Try playing file
mpv /music/file.xyz
# or
ffplay /music/file.xyz
```

**4. Supported formats**:
- MP3, FLAC, M4A, WAV, OGG (common)
- See [mb_identify_record.md](mb_identify_record.md#supported-formats) for full list

---

## Network Issues

### "Network error" / "Connection failed"

**Problem**: Can't reach MusicBrainz or AcoustID servers.

**Possible Causes**:
1. No internet connection
2. Server temporarily down
3. Firewall blocking requests
4. DNS resolution issues
5. Proxy configuration needed

**Solutions**:

**1. Check internet connection**:
```bash
ping -c 3 musicbrainz.org
ping -c 3 api.acoustid.org
```

**2. Try in browser**:
- https://musicbrainz.org/
- https://acoustid.org/

**3. Check firewall**:
```bash
# Linux: Check if port 443 (HTTPS) is allowed
sudo iptables -L | grep 443

# Allow if needed
sudo ufw allow out 443/tcp
```

**4. Check DNS**:
```bash
# Resolve hostname
nslookup musicbrainz.org

# Try different DNS server if needed
```

**5. Proxy settings** (if behind corporate firewall):
```bash
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080
```

**6. Wait and retry**:
Servers may be temporarily down for maintenance.

---

### Timeouts

**Problem**: Requests timeout without response.

**Causes**:
1. Slow internet connection
2. Server under heavy load
3. Large requests (full metadata)
4. Network instability

**Solutions**:

**1. Check connection speed**:
```bash
speedtest-cli
```

**2. Use lower metadata level**:
```json
// Timeout with full
{"file_path": "/music/track.mp3", "metadata_level": "full"}  // ❌

// Try minimal
{"file_path": "/music/track.mp3", "metadata_level": "minimal"}  // ✅
```

**3. Retry during off-peak hours**:
- UTC nights/weekends typically faster

**4. Simplify queries**:
```json
// Complex advanced search may timeout
{
  "entity": "release",
  "query": "artist:* AND date:[1900 TO 2024] AND format:* AND country:*"
}  // ❌ Too broad

// Simpler query
{
  "entity": "release",
  "query": "artist:Radiohead AND type:Album"
}  // ✅
```

---

## Rate Limiting Issues

### "Rate limit exceeded"

**Problem**: Too many requests in short time.

**Error**:
```
HTTP 429: Too Many Requests
Retry-After: 1
```

**Cause**: Exceeded MusicBrainz rate limit (1 request/second).

**Solutions**:

**1. Add delays**:
```rust
// Bad: Tight loop
for artist in artists {
    mb_artist_search(artist).await?;
}  // ❌ Will hit rate limits

// Good: With delays
for artist in artists {
    mb_artist_search(artist).await?;
    tokio::time::sleep(Duration::from_millis(1100)).await;
}  // ✅
```

**2. Use sequential testing**:
```bash
# Bad: Parallel tests
cargo test  # ❌

# Good: Sequential
cargo test -- --ignored --test-threads=1  # ✅
```

**3. Implement exponential backoff**:
```rust
let mut delay = Duration::from_secs(1);
for retry in 0..5 {
    match mb_artist_search(name).await {
        Ok(result) => return Ok(result),
        Err(_) => {
            tokio::time::sleep(delay).await;
            delay *= 2;  // 1s, 2s, 4s, 8s, 16s
        }
    }
}
```

**4. Cache results**:
Don't re-query the same data.

See [rate-limiting.md](rate-limiting.md) for detailed strategies.

---

## Data Quality Issues

### Incomplete Metadata

**Problem**: Results missing expected fields.

**Causes**:
1. Data not in MusicBrainz
2. Community hasn't added it yet
3. Used minimal metadata level

**Solutions**:

**1. Use higher metadata level**:
```json
{"file_path": "/music/track.mp3", "metadata_level": "full"}
```

**2. Check different releases**:
Some editions have more complete data:
```json
{
  "release": "OK Computer",
  "artist": "Radiohead",
  "find_release_group_versions": true
}
```

**3. Contribute to MusicBrainz**:
Help improve data quality:
https://musicbrainz.org/doc/How_to_Contribute

---

### Incorrect/Conflicting Data

**Problem**: Returned metadata doesn't match expectations.

**Causes**:
1. Multiple versions/editions
2. Data errors in MusicBrainz
3. Regional differences
4. Different master recordings

**Solutions**:

**1. Verify on website**:
Check data at: `https://musicbrainz.org/release/{mbid}`

**2. Compare multiple matches**:
```json
{
  "recording_mbid": "...",
  "find_appearances": true
}
```

**3. Report errors**:
If data is truly wrong, edit on MusicBrainz (requires account).

**4. Use release group versions**:
```json
{
  "release": "Album Name",
  "artist": "Artist",
  "find_release_group_versions": true,
  "limit": 50
}
```
Compare different editions to find correct one.

---

## System Issues

### Memory Issues (Large Batches)

**Problem**: System runs out of memory during bulk processing.

**Solutions**:

**1. Process in smaller batches**:
```rust
// Instead of all at once
let results = process_all_files(files).await;  // ❌

// Process in batches
for chunk in files.chunks(100) {
    let results = process_files(chunk).await;
    // Process/save results
    // Memory freed between batches
}  // ✅
```

**2. Stream results**:
Process and save results immediately instead of accumulating.

**3. Use minimal metadata**:
Less data per result = less memory.

---

### Permission Issues

**Problem**: Can't read audio files or write results.

**Solutions**:

**1. Check file permissions**:
```bash
ls -l /music/track.mp3
chmod +r /music/track.mp3  # Add read permission
```

**2. Check directory permissions**:
```bash
ls -ld /music/
chmod +rx /music/  # Add read and execute
```

**3. Run with appropriate user**:
Ensure user has access to music files.

---

## Testing Issues

### Tests Failing in CI

**Problem**: Tests pass locally but fail in CI/CD.

**Causes**:
1. Parallel test execution hitting rate limits
2. Network not available in CI
3. fpcalc not installed in CI environment

**Solutions**:

**1. Sequential tests in CI**:
```yaml
# .github/workflows/test.yml
- name: Run tests
  run: cargo test -- --ignored --test-threads=1
```

**2. Install dependencies**:
```yaml
- name: Install fpcalc
  run: sudo apt-get install libchromaprint-tools
```

**3. Mock network tests** (if appropriate):
Use test fixtures instead of real API calls.

---

## Getting Help

### When You're Stuck

1. **Search documentation**:
   - Check individual tool docs
   - Review [Common Concepts](common-concepts.md)
   - Read [Rate Limiting](rate-limiting.md)

2. **Check MusicBrainz**:
   - Visit https://musicbrainz.org/
   - Search for entity manually
   - Check if data exists

3. **Test with curl**:
   ```bash
   # Test MusicBrainz API directly
   curl "https://musicbrainz.org/ws/2/artist/?query=radiohead&fmt=json"
   ```

4. **Enable debug logging**:
   ```bash
   RUST_LOG=debug cargo run
   ```

5. **Community help**:
   - MusicBrainz forums: https://community.musicbrainz.org/
   - IRC: #musicbrainz on Libera.Chat
   - Project issues (if bug in this server)

---

## Common Error Messages Reference

| Error | Cause | Solution |
|-------|-------|----------|
| "No results found" | Search returned empty | Broaden search, check spelling |
| "MBID not found" | Invalid/deleted MBID | Verify format, search by name |
| "fpcalc not found" | Chromaprint not installed | Install libchromaprint-tools |
| "File not found" | Wrong path or permissions | Check path, fix permissions |
| "No matches found" | Not in AcoustID DB | Try manual search, check quality |
| "Network error" | No connection | Check internet, firewall |
| "Rate limit exceeded" | Too many requests | Add delays, use caching |
| "Timeout" | Slow connection/server | Simplify query, retry later |
| "Unsupported format" | Bad audio format | Convert to MP3/FLAC |

---

## See Also

- [Common Concepts](common-concepts.md) - Understand MusicBrainz terminology
- [Rate Limiting](rate-limiting.md) - Avoid rate limit issues
- Individual tool documentation - Tool-specific guidance
- [MusicBrainz Documentation](https://musicbrainz.org/doc) - Official docs
