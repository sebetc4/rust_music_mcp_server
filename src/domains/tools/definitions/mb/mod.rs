//! MusicBrainz tools module.
//!
//! This module provides domain-specific tools for searching the MusicBrainz database:
//! - `artist`: Search for artists and their releases
//! - `release`: Search for releases, tracks, and release group versions
//! - `recording`: Search for recordings and find where they appear
//! - `advanced`: Advanced Lucene-style queries across all entity types
//!
//! Each tool has handlers for both HTTP and STDIO/TCP transports.

pub mod advanced;
pub mod artist;
pub mod common;
pub mod identify_record;
pub mod recording;
pub mod release;

// Re-export domain-specific tools
pub use advanced::{MbAdvancedSearchParams, MbAdvancedSearchTool};
pub use artist::{MbArtistParams, MbArtistTool};
pub use identify_record::MbIdentifyRecordTool;
pub use recording::{MbRecordingParams, MbRecordingTool};
pub use release::{MbReleaseParams, MbReleaseTool};
