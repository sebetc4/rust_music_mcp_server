//! Common utilities shared across MusicBrainz tools.
//!
//! This module provides shared functionality like MBID validation,
//! response formatting, and error handling helpers.

use rmcp::model::{CallToolResult, Content};
use tracing::warn;

/// UUID format: 8-4-4-4-12 hexadecimal characters
const MBID_LENGTH: usize = 36;
const MBID_DASH_COUNT: usize = 4;

/// Check if a string looks like a MusicBrainz ID (UUID format).
///
/// MBIDs are UUIDs in the format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
/// Example: 5b11f4ce-a62d-471e-81fc-a69a8278c7da
pub fn is_mbid(query: &str) -> bool {
    query.len() == MBID_LENGTH
        && query.chars().filter(|c| *c == '-').count() == MBID_DASH_COUNT
        && query.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Format a duration in milliseconds to MM:SS format.
pub fn format_duration(length_ms: u64) -> String {
    let duration_secs = length_ms / 1000;
    let minutes = duration_secs / 60;
    let seconds = duration_secs % 60;
    format!("{}:{:02}", minutes, seconds)
}

/// Extract year from a date string.
/// MusicBrainz DateString format can be: "YYYY-MM-DD", "YYYY-MM", or "YYYY"
pub fn extract_year(date_str: &str) -> Option<String> {
    if date_str.len() >= 4 {
        Some(date_str[..4].to_string())
    } else {
        None
    }
}

/// Format a date string for display.
pub fn format_date(date_str: &str) -> String {
    date_str.to_string()
}

/// Create an error result with a formatted message.
pub fn error_result(message: &str) -> CallToolResult {
    warn!("{}", message);
    CallToolResult::error(vec![Content::text(message.to_string())])
}

/// Create a success result with text content.
pub fn success_result(content: String) -> CallToolResult {
    CallToolResult::success(vec![Content::text(content)])
}

/// Get artist name from artist credit.
pub fn get_artist_name(
    artist_credit: &Option<Vec<musicbrainz_rs::entity::artist_credit::ArtistCredit>>,
) -> String {
    artist_credit
        .as_ref()
        .and_then(|ac| ac.first())
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "Unknown Artist".to_string())
}

/// Default limit for search results.
pub fn default_limit() -> usize {
    10
}

/// Validate and clamp limit to allowed range (1-100).
pub fn validate_limit(limit: usize) -> usize {
    limit.min(100).max(1)
}

/// Common HTTP handler helper to extract entity parameter.
#[cfg(feature = "http")]
pub fn extract_entity_param(arguments: &serde_json::Value) -> Option<String> {
    arguments
        .get("entity")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mbid_valid() {
        assert!(is_mbid("5b11f4ce-a62d-471e-81fc-a69a8278c7da"));
        assert!(is_mbid("1b022e01-4da6-387b-8658-8678046e4cef"));
    }

    #[test]
    fn test_is_mbid_invalid() {
        assert!(!is_mbid("Nirvana"));
        assert!(!is_mbid("5b11f4ce-a62d-471e-81fc")); // too short
        assert!(!is_mbid("5b11f4ce-a62d-471e-81fc-a69a8278c7da-extra")); // too long
        assert!(!is_mbid("5b11f4ce_a62d_471e_81fc_a69a8278c7da")); // wrong separator
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(180000), "3:00");
        assert_eq!(format_duration(245000), "4:05");
        assert_eq!(format_duration(61000), "1:01");
        assert_eq!(format_duration(59000), "0:59");
    }

    #[test]
    fn test_validate_limit() {
        assert_eq!(validate_limit(10), 10);
        assert_eq!(validate_limit(0), 1);
        assert_eq!(validate_limit(200), 100);
        assert_eq!(validate_limit(50), 50);
    }

    #[test]
    fn test_extract_year() {
        assert_eq!(extract_year("1997-06-16"), Some("1997".to_string()));
        assert_eq!(extract_year("1997-06"), Some("1997".to_string()));
        assert_eq!(extract_year("1997"), Some("1997".to_string()));
        assert_eq!(extract_year("97"), None);
    }
}
