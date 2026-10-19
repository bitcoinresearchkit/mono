use std::{fs, io, result::Result as StdResult};

use thiserror::Error;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    TryLock(#[from] fs::TryLockError),

    #[error("Region not found")]
    RegionNotFound,

    #[error("Cannot remove region '{id}': still referenced ({ref_count} handles)")]
    RegionStillReferenced { id: String, ref_count: usize },

    #[error("Write position {position} is beyond region length {region_len}")]
    WriteOutOfBounds { position: usize, region_len: usize },

    #[error("Cannot truncate to {from} bytes (current length: {current_len})")]
    TruncateInvalid { from: usize, current_len: usize },

    #[error("Invalid region ID")]
    InvalidRegionId,

    #[error("Corrupted metadata: {0}")]
    CorruptedMetadata(String),

    #[error("Region size would overflow: current={current}, requested={requested}")]
    RegionSizeOverflow { current: usize, requested: usize },

    #[error("Database file size exceeds the addressable range: {requested}")]
    FileSizeOverflow { requested: usize },

    #[error(
        "Database file outgrew its reserved address space ({reserved} bytes, {requested} needed): restart to reserve more"
    )]
    ReservationExhausted { requested: usize, reserved: usize },

    #[error("Background task panicked")]
    BackgroundTaskPanicked,

    #[error("A background task cannot join itself")]
    BackgroundTaskSelfJoin,

    #[error("Failed to punch hole at offset {start} (length {len}): {source}")]
    HolePunchFailed {
        start: usize,
        len: usize,
        source: io::Error,
    },

    #[error("Hole punching is not supported on this platform")]
    HolePunchUnsupported,
}
