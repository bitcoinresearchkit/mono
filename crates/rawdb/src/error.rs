use std::{fs, io};

use thiserror::Error;

/// Result using rawdb's error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for rawdb operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error("Database is locked by another process")]
    TryLock(#[from] fs::TryLockError),

    // Region errors
    #[error("Region not found")]
    RegionNotFound,

    #[error("Region already exists")]
    RegionAlreadyExists,

    #[error("Cannot remove region '{id}': still held by {} reference(s)", ref_count - 1)]
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

    #[error("Invalid metadata size: expected {expected} bytes, got {actual}")]
    InvalidMetadataSize { expected: usize, actual: usize },

    #[error("Empty region metadata")]
    EmptyMetadata,

    // Layout errors
    #[error("Region index mismatch in layout")]
    RegionIndexMismatch,

    #[error("Hole too small: have {hole_size} bytes, need {requested}")]
    HoleTooSmall { hole_size: usize, requested: usize },

    // Internal invariant errors
    #[error("Internal invariant violated: {0}")]
    InvariantViolation(String),

    #[error("Corrupted metadata: {0}")]
    CorruptedMetadata(String),

    #[error("Region size would overflow: current={current}, requested={requested}")]
    RegionSizeOverflow { current: usize, requested: usize },

    #[error("Overlapping copy ranges not supported (src={src}..{src_end}, dst={dst}..{dst_end})")]
    OverlappingCopyRanges {
        src: usize,
        src_end: usize,
        dst: usize,
        dst_end: usize,
    },

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
    pub fn other(e: impl ToString) -> Self {
        Self::InvariantViolation(e.to_string())
    }
}
