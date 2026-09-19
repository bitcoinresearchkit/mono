use std::{fs, io, result::Result as StdResult};

use thiserror::Error;

/// Result using rawdb's error type.
pub type Result<T> = StdResult<T, Error>;

/// Error types for rawdb operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    TryLock(#[from] fs::TryLockError),

    // Region errors
    #[error("Region not found")]
    RegionNotFound,

    #[error("Cannot remove region '{id}': still referenced ({ref_count} handles)")]
    RegionStillReferenced { id: String, ref_count: usize },

    // Write errors
    #[error("Write position {position} is beyond region length {region_len}")]
    WriteOutOfBounds { position: usize, region_len: usize },

    // Truncate errors
    #[error("Cannot truncate to {from} bytes (current length: {current_len})")]
    TruncateInvalid { from: usize, current_len: usize },

    // Metadata errors
    #[error("Invalid region ID")]
    InvalidRegionId,

    /// An error reported by a caller-supplied operation.
    #[error("{0}")]
    Other(String),

    #[error("Corrupted metadata: {0}")]
    CorruptedMetadata(String),

    #[error("Region size would overflow: current={current}, requested={requested}")]
    RegionSizeOverflow { current: usize, requested: usize },

    #[error("Database file size exceeds the addressable range: {requested}")]
    FileSizeOverflow { requested: usize },

    #[error("Background task panicked")]
    BackgroundTaskPanicked,

    #[error("A background task cannot join itself")]
    BackgroundTaskSelfJoin,

    // Hole punching errors
    #[error("Failed to punch hole at offset {start} (length {len}): {source}")]
    HolePunchFailed {
        start: usize,
        len: usize,
        source: io::Error,
    },

    #[error("Hole punching is not supported on this platform")]
    HolePunchUnsupported,
}

impl Error {
    /// Preserves a caller-supplied error message without classifying its cause.
    pub fn other(e: impl ToString) -> Self {
        Self::Other(e.to_string())
    }
}
