# Rate Limiting

Guidelines and best practices for respecting MusicBrainz and AcoustID API rate limits.

---

## MusicBrainz API Limits

### Official Limits

- **Requests per second**: 1
- **Enforcement**: Server-side (429 Too Many Requests)
- **Applies to**: All MusicBrainz API endpoints

### Server Response

When rate limit exceeded:
```
HTTP 429 Too Many Requests
Retry-After: 1
```

### Automatic Handling

All MusicBrainz tools in this server automatically respect rate limits:
- Built-in delays between requests
- Automatic retry with backoff on 429 errors
- No manual rate limiting needed

---

## AcoustID API Limits

### Official Limits

- **No strict per-second limit**
- **Fair use policy**: Be reasonable
- **Typical limit**: ~3 requests/second sustained
- **Burst tolerance**: Higher for short periods

### Best Practices

1. **Don't hammer the API**: Space out requests
2. **Use caching**: Don't re-identify the same file
3. **Batch processing**: Add delays between files
4. **Monitor**: Watch for error responses

### Response on Abuse

- Temporary IP blocking
- API key suspension (if provided)
- Contact for resolution

---

## Testing with Rate Limits

### Problem

Running tests in parallel can trigger rate limits:
```bash
cargo test  # May fail due to parallel execution
```

### Solution

Use `--test-threads=1` for network tests:
```bash
cargo test -- --ignored --test-threads=1
```

This ensures:
- Tests run sequentially
- No concurrent API requests
- Rate limits are respected

### Test Organization

Tests are organized as:
- **Unit tests**: No network, run in parallel
- **Integration tests** (with `#[ignore]`): Network access, run sequentially

Run separately:
```bash
# Unit tests (fast, parallel)
cargo test

# Integration tests (slow, sequential)
cargo test -- --ignored --test-threads=1
```

---

## Batch Processing Guidelines

### Small Batches (< 100 items)

Sequential processing is fine:
```rust
for file in files {
    let result = mb_identify_record(file, "basic").await?;
    // Process result
    // Built-in rate limiting handles delays
}
```

### Medium Batches (100-1000 items)

Add explicit delays:
```rust
for file in files {
    let result = mb_identify_record(file, "minimal").await?;
    // Process result

    // Additional delay for safety
    tokio::time::sleep(Duration::from_millis(1500)).await;
}
```

### Large Batches (1000+ items)

Use queuing with rate limiter:
```rust
use tokio::time::{interval, Duration};

let mut interval = interval(Duration::from_millis(1200));

for file in files {
    interval.tick().await;  // Wait for next slot

    let result = mb_identify_record(file, "minimal").await?;
    // Process result
}
```

---

## Best Practices

### 1. Cache Results

**Don't re-query the same data**:
```rust
// Bad: Query same artist repeatedly
for album in albums {
    let artist = mb_artist_search(artist_name).await?;  // Wasteful!
}

// Good: Query once, cache result
let artist = mb_artist_search(artist_name).await?;
for album in albums {
    // Use cached artist data
}
```

### 2. Use MBIDs for Repeated Lookups

**MBIDs are faster and use fewer resources**:
```rust
// First lookup (search by name)
let artist = mb_artist_search("Radiohead").await?;
let mbid = artist.mbid;

// Subsequent lookups (direct by MBID - faster)
let artist = mb_artist_search(mbid).await?;
```

### 3. Batch Operations During Off-Peak Hours

**For very large operations**:
- Run during off-peak (UTC nights/weekends)
- Reduces server load
- Better for community

### 4. Don't Hammer the API

**Respect the infrastructure**:
```rust
// Bad: Tight loop, no delays
for _ in 0..1000 {
    mb_artist_search(name).await?;  // Will hit rate limits!
}

// Good: Proper spacing
for _ in 0..1000 {
    mb_artist_search(name).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;  // Respect limits
}
```

### 5. Implement Request Queuing

**For production applications**:
- Use request queue with rate limiter
- Process requests in order
- Automatic retry on failures

### 6. Use Minimal Metadata Level

**For bulk identification**:
```rust
// Faster, less data transfer
mb_identify_record(file, "minimal").await?;

// Only use "full" when truly needed
mb_identify_record(file, "full").await?;  // Slower, more API calls
```

---

## Rate Limit Error Handling

### Detection

```rust
match result {
    Err(e) if e.to_string().contains("429") => {
        // Rate limit exceeded
        eprintln!("Rate limit hit, waiting...");
        tokio::time::sleep(Duration::from_secs(2)).await;
        // Retry
    }
    Err(e) => return Err(e),
    Ok(data) => // Process data
}
```

### Exponential Backoff

For robust applications:
```rust
let mut retry_delay = Duration::from_secs(1);
let max_retries = 5;

for attempt in 0..max_retries {
    match mb_artist_search(name).await {
        Ok(result) => return Ok(result),
        Err(e) if e.to_string().contains("429") => {
            eprintln!("Rate limit (attempt {}), waiting {:?}", attempt + 1, retry_delay);
            tokio::time::sleep(retry_delay).await;
            retry_delay *= 2;  // Exponential backoff
        }
        Err(e) => return Err(e),
    }
}
```

---

## Monitoring and Logging

### Request Logging

Track API usage:
```rust
let start = Instant::now();
let result = mb_artist_search(name).await?;
let duration = start.elapsed();

info!("MusicBrainz request completed in {:?}", duration);
```

### Rate Limit Tracking

For production systems:
- Count requests per minute/hour
- Alert on approaching limits
- Throttle when necessary

### Metrics to Track

- **Requests per minute**: Should stay ≤ 60 for MusicBrainz
- **Average response time**: Typical: 200-500ms
- **429 error rate**: Should be 0% or very low
- **Cache hit rate**: Higher is better

---

## API Key Best Practices

### AcoustID API Key

While not strictly required, using an API key:
- ✅ Higher rate limits
- ✅ Better error handling
- ✅ Usage tracking
- ✅ Support priority

### Configuration

Set via environment:
```bash
export ACOUSTID_API_KEY="your-key-here"
```

### Getting a Key

1. Visit: https://acoustid.org/api-key
2. Register (free for non-commercial use)
3. Add key to environment/config

---

## Special Considerations

### MusicBrainz Identification

**Fingerprinting is expensive**:
- Generates audio fingerprint (CPU intensive)
- Queries AcoustID
- May query MusicBrainz multiple times for full metadata

**Recommendations**:
- Use `metadata_level: "minimal"` for bulk operations
- Cache fingerprints (store `fpcalc` output)
- Don't re-identify the same files

### CI/CD Testing

**In continuous integration**:
```yaml
# .github/workflows/test.yml
- name: Run network tests
  run: cargo test -- --ignored --test-threads=1
  env:
    RUST_TEST_THREADS: 1
```

Ensures tests don't fail due to rate limits.

---

## Troubleshooting Rate Limits

### "Rate limit exceeded" Errors

**Symptoms**:
- HTTP 429 responses
- "Too many requests" errors
- Timeouts

**Solutions**:
1. Add delays between requests (1+ seconds)
2. Use `--test-threads=1` for tests
3. Implement exponential backoff
4. Check for request loops

### Slow Response Times

**Symptoms**:
- Requests taking > 2 seconds
- Timeouts

**Possible causes**:
1. Server under load (peak hours)
2. Complex queries (advanced search)
3. Network latency
4. Rate limiting (throttled responses)

**Solutions**:
- Retry during off-peak hours
- Simplify queries
- Use caching
- Check network connection

### Tests Failing Randomly

**Symptoms**:
- Tests pass individually
- Fail when run together
- "429" errors in test output

**Solution**:
```bash
# Always run network tests sequentially
cargo test -- --ignored --test-threads=1
```

---

## Summary

### Key Takeaways

1. **MusicBrainz**: 1 request/second limit, strictly enforced
2. **AcoustID**: Fair use, ~3 requests/second sustained
3. **Testing**: Always use `--test-threads=1` for network tests
4. **Caching**: Essential for efficiency
5. **MBIDs**: Use for repeated lookups (faster)
6. **Batch Processing**: Add delays, use minimal metadata
7. **Error Handling**: Implement retry with backoff

### Quick Reference

| Operation | Recommended Delay |
|-----------|-------------------|
| MusicBrainz search | 1.0s (automatic) |
| AcoustID lookup | 0.5s |
| Batch processing | 1.5s |
| After 429 error | 2.0s (then exponential) |

---

## External Resources

- [MusicBrainz API Rate Limiting](https://musicbrainz.org/doc/MusicBrainz_API/Rate_Limiting)
- [AcoustID API Documentation](https://acoustid.org/webservice)
- [MusicBrainz Code of Conduct](https://musicbrainz.org/doc/Code_of_Conduct/API)

---

## See Also

- [Common Concepts](common-concepts.md) - MBIDs and caching strategies
- [Troubleshooting](troubleshooting.md) - Error handling
- Individual tool documentation for specific usage patterns
