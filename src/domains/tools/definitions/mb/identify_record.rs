//! MusicBrainz audio identification tool using AcoustID/Chromaprint.
//!
//! This tool identifies audio files by their acoustic fingerprint, even when
//! metadata is missing or incorrect (e.g., files downloaded from YouTube).

use futures::FutureExt;
use rmcp::{
    ErrorData as McpError,
    handler::server::tool::{ToolCallContext, ToolRoute, cached_schema_for_type},
    model::{CallToolResult, Content, Tool},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

use crate::core::config::Config;
use crate::core::security::validate_path;

// ============================================================================
// Configuration & Constants
// ============================================================================

const ACOUSTID_API_URL: &str = "https://api.acoustid.org/v2/lookup";
const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 1000;
const REQUEST_TIMEOUT_SECS: u64 = 30;
const MAX_RESULT_LIMIT: usize = 10;

// ============================================================================
// Tool Parameters
// ============================================================================

/// Metadata detail level for AcoustID API responses.
///
/// Controls how much information is retrieved from the AcoustID database.
/// Higher levels provide more data but may take slightly longer to process.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[schemars(rename_all = "lowercase")]
pub enum MetadataLevel {
    /// Only MusicBrainz recording IDs (fastest, use when you only need IDs for further queries)
    Minimal,
    /// Recording IDs with title, artists, and duration (recommended for most cases)
    Basic,
    /// Complete metadata including release groups, albums, formats, and dates
    Full,
}

impl MetadataLevel {
    /// Convert to AcoustID API meta parameter value.
    ///
    /// Based on real API testing:
    /// - "recordingids" returns only IDs
    /// - "recordings" returns IDs + title + artists + duration
    /// - "recordings releasegroups compress" returns everything + album info
    fn as_api_param(self) -> &'static str {
        match self {
            Self::Minimal => "recordingids",
            Self::Basic => "recordings",
            Self::Full => "recordings releasegroups compress",
        }
    }
}

impl Default for MetadataLevel {
    fn default() -> Self {
        Self::Basic
    }
}

/// Parameters for the audio identification tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MbIdentifyRecordParams {
    /// Path to the audio file to identify
    pub file_path: String,

    /// Maximum number of results to return (default: 3, max: 10)
    #[serde(default = "default_result_limit")]
    pub limit: usize,

    /// Metadata detail level (default: basic)
    #[serde(default)]
    pub metadata_level: MetadataLevel,
}

fn default_result_limit() -> usize {
    3
}

// ============================================================================
// AcoustID API Response Structures
// ============================================================================

#[derive(Debug, Deserialize)]
struct AcoustIDResponse {
    status: String,
    #[serde(default)]
    results: Vec<AcoustIDResult>,
    error: Option<AcoustIDError>,
}

#[derive(Debug, Deserialize)]
struct AcoustIDResult {
    id: String,
    score: f64,
    #[serde(default)]
    recordings: Vec<AcoustIDRecording>,
}

#[derive(Debug, Deserialize)]
struct AcoustIDRecording {
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    artists: Vec<AcoustIDArtist>,
    #[serde(default)]
    releases: Vec<AcoustIDRelease>,
    #[serde(default)]
    releasegroups: Vec<AcoustIDReleaseGroup>,
}

#[derive(Debug, Clone, Deserialize)]
struct AcoustIDArtist {
    name: String,
    #[allow(dead_code)]
    #[serde(default)]
    id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AcoustIDReleaseGroup {
    #[allow(dead_code)]
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    artists: Vec<AcoustIDArtist>,
    #[serde(default)]
    releases: Vec<AcoustIDRelease>,
}

#[derive(Debug, Deserialize)]
struct AcoustIDRelease {
    #[allow(dead_code)]
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    date: Option<AcoustIDDate>,
    #[serde(default)]
    track_count: Option<u32>,
    #[allow(dead_code)]
    #[serde(default)]
    medium_count: Option<u32>,
    #[serde(default)]
    mediums: Vec<AcoustIDMedium>,
}

#[derive(Debug, Deserialize)]
struct AcoustIDMedium {
    #[allow(dead_code)]
    #[serde(default)]
    position: Option<u32>,
    #[serde(default)]
    format: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    track_count: Option<u32>,
    #[serde(default)]
    tracks: Vec<AcoustIDTrack>,
}

#[derive(Debug, Deserialize)]
struct AcoustIDTrack {
    #[allow(dead_code)]
    #[serde(default)]
    id: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    position: Option<u32>,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AcoustIDDate {
    year: Option<u32>,
    month: Option<u32>,
    day: Option<u32>,
}

impl AcoustIDDate {
    /// Format date in ISO 8601 format (YYYY-MM-DD, YYYY-MM, or YYYY).
    fn format(&self) -> String {
        match (self.year, self.month, self.day) {
            (Some(y), Some(m), Some(d)) => format!("{y:04}-{m:02}-{d:02}"),
            (Some(y), Some(m), None) => format!("{y:04}-{m:02}"),
            (Some(y), None, None) => format!("{y:04}"),
            _ => "Unknown".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AcoustIDError {
    message: String,
}

// ============================================================================
// Helper Structures
// ============================================================================

#[derive(Debug)]
struct FingerprintData {
    duration: u32,
    fingerprint: String,
}

#[derive(Debug, Deserialize)]
struct FpcalcOutput {
    duration: f64,
    fingerprint: String,
}

// ============================================================================
// Custom Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
enum IdentificationError {
    #[error("Audio file not found or inaccessible: {0}")]
    FileNotFound(String),

    #[error("Chromaprint (fpcalc) is not installed.\n{0}")]
    FpcalcNotFound(String),

    #[error("Failed to generate audio fingerprint: {0}")]
    FingerprintFailed(String),

    #[error("AcoustID API request failed: {0}")]
    ApiError(String),

    #[error(
        "No matches found in AcoustID database.\n\
             This audio file may not be indexed yet, or the quality might be too low for accurate fingerprinting.\n\
             Try:\n  \
             - Ensuring the file is not corrupted\n  \
             - Using a higher quality source\n  \
             - Checking if this is a very obscure or unreleased recording"
    )]
    NoMatches,

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error(
        "AcoustID API key is invalid or expired.\n\
            The default public key is no longer valid or has exceeded its rate limits.\n\
            Please set your own API key via environment variable: MCP_ACOUSTID_API_KEY\n\
            You can request a free API key at: https://acoustid.org/api-key"
    )]
    InvalidApiKey,
}

// ============================================================================
// Tool Definition
// ============================================================================

/// MusicBrainz audio identification tool.
pub struct MbIdentifyRecordTool;

impl MbIdentifyRecordTool {
    /// Tool name as registered in MCP.
    pub const NAME: &'static str = "mb_identify_record";

    /// Tool description shown to clients.
    pub const DESCRIPTION: &'static str = "Identify audio files using acoustic fingerprinting via AcoustID/Chromaprint.\n\
         \n\
         Ideal for identifying songs when:\n\
         - Metadata is missing, incorrect, or incomplete\n\
         - Files were downloaded from YouTube, SoundCloud, or other streaming platforms\n\
         - Files were converted from video to audio formats\n\
         - Files have been renamed or lack proper ID3 tags\n\
         \n\
         This tool analyzes the actual audio waveform and matches it against the AcoustID database\n\
         (linked to MusicBrainz). Returns detailed metadata including MusicBrainz Recording IDs,\n\
         artist names, album information, and more.\n\
         \n\
         Supports all common audio formats: MP3, FLAC, WAV, OGG, M4A, AAC, WMA, OPUS, and more.";

    /// Execute the tool logic.
    #[instrument(skip_all, fields(file_path = %params.file_path, limit = params.limit))]
    pub fn execute(params: &MbIdentifyRecordParams, config: &Config) -> CallToolResult {
        info!("Starting audio identification");

        // Get API key from config (always present due to default)
        let api_key = config
            .credentials
            .acoustid_api_key
            .as_deref()
            .unwrap_or_default();

        match Self::identify_audio_internal(params, api_key, config) {
            Ok(content) => {
                info!("Audio identification completed successfully");
                CallToolResult::success(vec![Content::text(content)])
            }
            Err(e) => {
                error!("Audio identification failed: {}", e);
                CallToolResult::error(vec![Content::text(e.to_string())])
            }
        }
    }

    /// Internal identification logic with proper error handling.
    fn identify_audio_internal(
        params: &MbIdentifyRecordParams,
        api_key: &str,
        config: &Config,
    ) -> Result<String, IdentificationError> {
        // Validate path security first
        validate_path(&params.file_path, config).map_err(|e| {
            IdentificationError::FileNotFound(format!("Path security validation failed: {}", e))
        })?;

        // Validate file exists and is accessible
        Self::validate_file(&params.file_path)?;

        // Validate and clamp limit
        let limit = params.limit.clamp(1, MAX_RESULT_LIMIT);

        // Generate fingerprint
        let fingerprint_data = Self::generate_fingerprint(&params.file_path)?;

        // Query API
        let response = Self::query_acoustid(api_key, &fingerprint_data, params.metadata_level)?;

        // Format results
        Self::format_results(&response, &params.file_path, limit, &params.metadata_level)
    }

    /// Validate that the file exists and is accessible.
    fn validate_file(file_path: &str) -> Result<(), IdentificationError> {
        let path = Path::new(file_path);

        if !path.exists() {
            return Err(IdentificationError::FileNotFound(file_path.to_string()));
        }

        if !path.is_file() {
            return Err(IdentificationError::FileNotFound(format!(
                "{file_path} is not a regular file (it may be a directory or symlink)"
            )));
        }

        // Check if file is readable
        if let Err(e) = std::fs::metadata(path) {
            return Err(IdentificationError::FileNotFound(format!(
                "Cannot access {file_path}: {e}"
            )));
        }

        Ok(())
    }

    /// Generate audio fingerprint using fpcalc command-line tool.
    #[instrument(skip_all, fields(file = %file_path))]
    fn generate_fingerprint(file_path: &str) -> Result<FingerprintData, IdentificationError> {
        // Check if fpcalc is installed
        if !Self::is_fpcalc_installed() {
            return Err(IdentificationError::FpcalcNotFound(
                "Installation instructions:\n\
                 • Linux (Debian/Ubuntu): sudo apt-get install libchromaprint-tools\n\
                 • Linux (Fedora/RHEL):   sudo dnf install chromaprint-tools\n\
                 • macOS:                 brew install chromaprint\n\
                 • Windows:               Download from https://acoustid.org/chromaprint\n\
                 \nAfter installation, verify with: fpcalc -version"
                    .to_string(),
            ));
        }

        debug!("Running fpcalc on {}", file_path);

        // Run fpcalc to generate fingerprint
        let output = Command::new("fpcalc")
            .arg("-json")
            .arg(file_path)
            .output()
            .map_err(|e| {
                IdentificationError::FingerprintFailed(format!("Failed to run fpcalc: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            return Err(IdentificationError::FingerprintFailed(stderr));
        }

        // Parse output directly from bytes (more efficient than converting to string first)
        let fpcalc_output: FpcalcOutput = serde_json::from_slice(&output.stdout).map_err(|e| {
            IdentificationError::FingerprintFailed(format!("Invalid JSON output: {e}"))
        })?;

        let duration = fpcalc_output.duration as u32;
        let fingerprint = fpcalc_output.fingerprint;

        debug!(
            "Generated fingerprint: duration={duration}s, len={}",
            fingerprint.len()
        );

        Ok(FingerprintData {
            duration,
            fingerprint,
        })
    }

    /// Check if fpcalc is installed on the system.
    fn is_fpcalc_installed() -> bool {
        Command::new("fpcalc").arg("-version").output().is_ok()
    }

    /// Query the AcoustID API with the fingerprint.
    #[instrument(skip(fingerprint_data), fields(duration = fingerprint_data.duration, metadata_level = ?metadata_level))]
    fn query_acoustid(
        api_key: &str,
        fingerprint_data: &FingerprintData,
        metadata_level: MetadataLevel,
    ) -> Result<AcoustIDResponse, IdentificationError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|e| {
                IdentificationError::ApiError(format!("Failed to create HTTP client: {}", e))
            })?;

        let mut last_error = String::new();

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let delay = BASE_DELAY_MS * 2u64.pow(attempt - 1);
                debug!(
                    "Retrying (attempt {}/{}) after {}ms",
                    attempt + 1,
                    MAX_RETRIES,
                    delay
                );
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }

            match Self::try_api_request(&client, api_key, fingerprint_data, metadata_level) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // Don't retry on API key errors - fail fast
                    if matches!(e, IdentificationError::InvalidApiKey) {
                        return Err(e);
                    }
                    last_error = e.to_string();
                    warn!("API request attempt {} failed: {}", attempt + 1, last_error);
                }
            }
        }

        Err(IdentificationError::ApiError(format!(
            "All {} retry attempts failed. Last error: {}\n\n\
             Troubleshooting:\n\
             • Check AcoustID service status: https://acoustid.org/\n\
             • Verify network connectivity (can you reach acoustid.org?)\n\
             • Rate limit exceeded? Wait 2-5 minutes before retrying\n\
             • Firewall or proxy blocking api.acoustid.org?\n\
             • For persistent issues, try using a custom API key",
            MAX_RETRIES, last_error
        )))
    }

    /// Attempt a single API request.
    fn try_api_request(
        client: &reqwest::blocking::Client,
        api_key: &str,
        fingerprint_data: &FingerprintData,
        metadata_level: MetadataLevel,
    ) -> Result<AcoustIDResponse, IdentificationError> {
        // Pre-format duration string to avoid allocation in form builder
        let duration_str = fingerprint_data.duration.to_string();

        let response = client
            .post(ACOUSTID_API_URL)
            .form(&[
                ("client", api_key),
                ("duration", duration_str.as_str()),
                ("fingerprint", fingerprint_data.fingerprint.as_str()),
                ("meta", metadata_level.as_api_param()),
            ])
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    IdentificationError::ApiError("Request timed out".to_string())
                } else if e.is_connect() {
                    IdentificationError::ApiError("Connection failed".to_string())
                } else {
                    IdentificationError::ApiError(e.to_string())
                }
            })?;

        let status = response.status();

        // Handle rate limiting
        if status.as_u16() == 429 {
            return Err(IdentificationError::ApiError(
                "Rate limit exceeded".to_string(),
            ));
        }

        // Handle server errors (retryable)
        if status.is_server_error() {
            return Err(IdentificationError::ApiError(format!(
                "Server error ({})",
                status
            )));
        }

        // Handle client errors (not retryable)
        if status.is_client_error() {
            // Specific handling for authentication errors (invalid API key)
            if status.as_u16() == 400 || status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(IdentificationError::InvalidApiKey);
            }
            return Err(IdentificationError::ApiError(format!(
                "Client error ({status}): Check your request parameters"
            )));
        }

        // Parse response directly from bytes for better performance
        let response_bytes = response
            .bytes()
            .map_err(|e| IdentificationError::InvalidResponse(e.to_string()))?;

        debug!("API response received: {} bytes", response_bytes.len());

        let acoustid_response: AcoustIDResponse = serde_json::from_slice(&response_bytes)
            .map_err(|e| IdentificationError::InvalidResponse(format!("JSON parse error: {e}")))?;

        // Check for API-level errors
        if acoustid_response.status != "ok" {
            if let Some(error) = acoustid_response.error {
                return Err(IdentificationError::ApiError(error.message));
            }
            return Err(IdentificationError::ApiError(
                "Unknown API error".to_string(),
            ));
        }

        Ok(acoustid_response)
    }

    /// Helper to format duration from seconds.
    fn format_duration(seconds: u32) -> String {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{minutes}:{secs:02}")
    }

    /// Helper to format release info.
    fn format_release_info(output: &mut String, release: &AcoustIDRelease) {
        if let Some(country) = &release.country {
            write!(output, "       Country: {country}").unwrap();
        }
        if let Some(date) = &release.date {
            write!(output, " | Date: {}", date.format()).unwrap();
        }
        writeln!(output).unwrap();

        if let Some(track_count) = release.track_count {
            writeln!(output, "       Tracks: {track_count}").unwrap();
        }
    }

    /// Format the results into a readable string.
    fn format_results(
        response: &AcoustIDResponse,
        file_path: &str,
        limit: usize,
        metadata_level: &MetadataLevel,
    ) -> Result<String, IdentificationError> {
        if response.results.is_empty() {
            return Err(IdentificationError::NoMatches);
        }

        let mut output = String::with_capacity(2048); // Pre-allocate for better performance

        writeln!(&mut output, "=== Audio Fingerprint Analysis ===").unwrap();
        writeln!(&mut output, "File: {file_path}").unwrap();
        writeln!(&mut output, "Metadata Level: {metadata_level:?}").unwrap();
        writeln!(
            &mut output,
            "\nFound {} potential match(es)\n",
            response.results.len()
        )
        .unwrap();

        let mut match_count = 0;

        for (i, result) in response.results.iter().take(limit).enumerate() {
            let confidence = (result.score * 100.0) as u32;

            writeln!(
                &mut output,
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            )
            .unwrap();
            writeln!(&mut output, "Match #{} (Confidence: {confidence}%)", i + 1).unwrap();
            writeln!(&mut output, "AcoustID: {}", result.id).unwrap();

            if result.recordings.is_empty() {
                writeln!(
                    &mut output,
                    "⚠ No MusicBrainz recordings linked to this AcoustID"
                )
                .unwrap();
            } else {
                writeln!(&mut output, "\nMusicBrainz Recording(s):").unwrap();
                for (rec_idx, recording) in result.recordings.iter().enumerate() {
                    writeln!(&mut output, "\n  Recording #{}", rec_idx + 1).unwrap();
                    writeln!(&mut output, "  ID: {}", recording.id).unwrap();

                    // Display metadata based on level
                    match metadata_level {
                        MetadataLevel::Minimal => {
                            // Only ID is shown (already displayed above)
                        }
                        MetadataLevel::Basic | MetadataLevel::Full => {
                            // Try to get title and artists from releasegroups first, fallback to recording fields
                            let (title_opt, artists_vec) = if !recording.releasegroups.is_empty() {
                                let rg = &recording.releasegroups[0];

                                // Extract track title from the first release's first medium's first track
                                let track_title = rg
                                    .releases
                                    .first()
                                    .and_then(|rel| rel.mediums.first())
                                    .and_then(|med| med.tracks.first())
                                    .and_then(|track| track.title.as_ref())
                                    .map(|s| s.to_string());

                                // Use releasegroup artists if available, otherwise recording artists
                                let artists = if !rg.artists.is_empty() {
                                    rg.artists.clone()
                                } else {
                                    recording.artists.clone()
                                };

                                (track_title.or_else(|| recording.title.clone()), artists)
                            } else {
                                (recording.title.clone(), recording.artists.clone())
                            };

                            if let Some(title) = &title_opt {
                                writeln!(&mut output, "  Title: {title}").unwrap();
                            }

                            if let Some(duration) = recording.duration {
                                let formatted = Self::format_duration(duration as u32);
                                writeln!(&mut output, "  Duration: {formatted}").unwrap();
                            }

                            if !artists_vec.is_empty() {
                                let artist_names: Vec<_> =
                                    artists_vec.iter().map(|a| a.name.as_str()).collect();
                                writeln!(&mut output, "  Artist(s): {}", artist_names.join(", "))
                                    .unwrap();
                            }

                            // Full metadata includes releasegroups and releases
                            if matches!(metadata_level, MetadataLevel::Full) {
                                // Show releasegroups first (contains album info)
                                if !recording.releasegroups.is_empty() {
                                    writeln!(&mut output, "\n  Release Groups:").unwrap();
                                    for (rg_idx, rg) in
                                        recording.releasegroups.iter().take(3).enumerate()
                                    {
                                        writeln!(
                                            &mut output,
                                            "    {}. {} ({})",
                                            rg_idx + 1,
                                            rg.title.as_deref().unwrap_or("Untitled"),
                                            rg.r#type.as_deref().unwrap_or("Unknown")
                                        )
                                        .unwrap();

                                        if let Some(rel) = rg.releases.first() {
                                            Self::format_release_info(&mut output, rel);

                                            if let Some(format) =
                                                &rel.mediums.first().and_then(|m| m.format.as_ref())
                                            {
                                                writeln!(&mut output, "       Format: {format}")
                                                    .unwrap();
                                            }
                                        }
                                    }
                                    if recording.releasegroups.len() > 3 {
                                        writeln!(
                                            &mut output,
                                            "    ... and {} more release group(s)",
                                            recording.releasegroups.len() - 3
                                        )
                                        .unwrap();
                                    }
                                }

                                // Also show direct releases if available
                                if !recording.releases.is_empty()
                                    && recording.releasegroups.is_empty()
                                {
                                    writeln!(&mut output, "\n  Releases:").unwrap();
                                    for (rel_idx, release) in
                                        recording.releases.iter().take(3).enumerate()
                                    {
                                        writeln!(
                                            &mut output,
                                            "    {}. {}",
                                            rel_idx + 1,
                                            release.title.as_deref().unwrap_or("Untitled")
                                        )
                                        .unwrap();

                                        Self::format_release_info(&mut output, release);
                                    }

                                    if recording.releases.len() > 3 {
                                        writeln!(
                                            &mut output,
                                            "    ... and {} more release(s)",
                                            recording.releases.len() - 3
                                        )
                                        .unwrap();
                                    }
                                }
                            }
                        }
                    }

                    match_count += 1;
                }
            }
            writeln!(&mut output).unwrap();
        }

        writeln!(
            &mut output,
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        )
        .unwrap();

        if match_count > 0 {
            match metadata_level {
                MetadataLevel::Minimal => {
                    writeln!(
                        &mut output,
                        "✓ Next step: Use 'mb_record_search' with one of the Recording IDs above"
                    )
                    .unwrap();
                    writeln!(
                        &mut output,
                        "  to retrieve full metadata (title, artist, album, etc.)"
                    )
                    .unwrap();
                }
                MetadataLevel::Basic => {
                    writeln!(
                        &mut output,
                        "✓ Basic metadata retrieved. For more details (releases, labels, etc.):"
                    )
                    .unwrap();
                    writeln!(
                        &mut output,
                        "  - Use 'metadata_level: full' for complete information"
                    )
                    .unwrap();
                    writeln!(
                        &mut output,
                        "  - Or use 'mb_record_search' with a Recording ID"
                    )
                    .unwrap();
                }
                MetadataLevel::Full => {
                    writeln!(
                        &mut output,
                        "✓ Full metadata retrieved from AcoustID database"
                    )
                    .unwrap();
                    writeln!(
                        &mut output,
                        "  Use 'mb_record_search' for additional MusicBrainz details if needed"
                    )
                    .unwrap();
                }
            }
        } else {
            writeln!(
                &mut output,
                "⚠ Fingerprint matched but no MusicBrainz data available."
            )
            .unwrap();
            writeln!(
                &mut output,
                "  This audio may not be in the MusicBrainz database yet."
            )
            .unwrap();
        }

        Ok(output)
    }

    /// HTTP handler for this tool (for HTTP transport).
    #[cfg(feature = "http")]
    pub fn http_handler(
        arguments: serde_json::Value,
        config: Arc<Config>,
    ) -> Result<serde_json::Value, String> {
        let params: MbIdentifyRecordParams =
            serde_json::from_value(arguments).map_err(|e| format!("Invalid parameters: {}", e))?;

        info!(
            "Audio identification tool (HTTP) called for: {}",
            params.file_path
        );

        let handle = std::thread::spawn(move || Self::execute(&params, &config));

        let result = handle
            .join()
            .map_err(|_| "Identification thread panicked".to_string())?;

        Ok(serde_json::json!({
            "content": result.content,
            "isError": result.is_error.unwrap_or(false)
        }))
    }

    /// Create a Tool model for this tool (metadata).
    pub fn to_tool() -> Tool {
        Tool {
            name: Self::NAME.into(),
            description: Some(Self::DESCRIPTION.into()),
            input_schema: cached_schema_for_type::<MbIdentifyRecordParams>(),
            annotations: None,
            output_schema: None,
            icons: None,
            meta: None,
            title: None,
        }
    }

    /// Create a ToolRoute for STDIO/TCP transport.
    pub fn create_route<S>(config: Arc<Config>) -> ToolRoute<S>
    where
        S: Send + Sync + 'static,
    {
        ToolRoute::new_dyn(Self::to_tool(), move |ctx: ToolCallContext<'_, S>| {
            let args = ctx.arguments.clone().unwrap_or_default();
            let config = config.clone();
            async move {
                let params: MbIdentifyRecordParams =
                    serde_json::from_value(serde_json::Value::Object(args))
                        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

                let result = tokio::task::spawn_blocking(move || Self::execute(&params, &config))
                    .await
                    .map_err(|e| {
                        McpError::internal_error(format!("Task execution failed: {}", e), None)
                    })?;

                Ok(result)
            }
            .boxed()
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_not_found() {
        let result = MbIdentifyRecordTool::validate_file("/nonexistent/file.mp3");
        assert!(matches!(result, Err(IdentificationError::FileNotFound(_))));
    }

    #[test]
    fn test_default_limit() {
        assert_eq!(default_result_limit(), 3);
    }

    #[test]
    fn test_params_deserialization() {
        let json = r#"{"file_path": "test.mp3"}"#;
        let params: MbIdentifyRecordParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.file_path, "test.mp3");
        assert_eq!(params.limit, 3);
    }

    #[test]
    fn test_mb_identify_missing_file() {
        let config = Config::default();
        let params = MbIdentifyRecordParams {
            file_path: "/nonexistent/file.mp3".to_string(),
            limit: 3,
            metadata_level: MetadataLevel::Basic,
        };

        let result = MbIdentifyRecordTool::execute(&params, &config);
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    #[ignore = "requires test audio file"]
    fn test_mb_identify_integration() {
        let config = Config::default();
        let test_file = "test_audio.mp3";

        if !std::path::Path::new(test_file).exists() {
            println!("Skipping integration test: test file not found");
            return;
        }

        let params = MbIdentifyRecordParams {
            file_path: test_file.to_string(),
            limit: 3,
            metadata_level: MetadataLevel::Basic,
        };

        let result = MbIdentifyRecordTool::execute(&params, &config);
        assert!(!result.content.is_empty());
    }

    #[cfg(feature = "http")]
    #[test]
    fn test_mb_identify_http_handler_invalid_params() {
        let config = Arc::new(Config::default());
        let args = serde_json::json!({
            "limit": 3
            // missing file_path
        });

        let result = MbIdentifyRecordTool::http_handler(args, config);
        assert!(result.is_err());
    }
}
