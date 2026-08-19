use std::{fmt, fs, io, time};

use thiserror::Error;

#[cfg(feature = "zerocopy")]
mod zerocopy;

use crate::{Format, Stamp, Version};

/// Result using vecdb's error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for vecdb operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Format(#[from] fmt::Error),
    #[error("Couldn't lock file. It must be already opened by another process.")]
    TryLockError(#[from] fs::TryLockError),
    #[cfg(feature = "zerocopy")]
    #[error("ZeroCopy error")]
    ZeroCopyError,
    #[error(transparent)]
    SystemTimeError(#[from] time::SystemTimeError),
    #[cfg(feature = "pco")]
    #[error(transparent)]
    PCO(#[from] pco::errors::PcoError),
    #[cfg(feature = "lz4")]
    #[error(transparent)]
    LZ4(#[from] lz4_flex::block::DecompressError),
    #[error(transparent)]
    RawDB(#[from] rawdb::Error),
    #[cfg(feature = "serde_json")]
    #[error(transparent)]
    SerdeJSON(#[from] serde_json::Error),
    #[cfg(feature = "sonic-rs")]
    #[error(transparent)]
    SonicRS(#[from] sonic_rs::Error),

    #[error("Wrong length: received: {received:?}, expected: {expected:?}")]
    WrongLength { received: usize, expected: usize },
    #[error("Wrong endian")]
    WrongEndian,
    #[error("Iterator ended")]
    IteratorEnded,
    #[error("Different version received: {received:?}, expected: {expected:?}")]
    DifferentVersion {
        received: Version,
        expected: Version,
    },
    #[error("Index too high: index: {index}, len: {len}, name: {name}")]
    IndexTooHigh {
        index: usize,
        len: usize,
        name: String,
    },
    #[error("Unexpected index: expected {expected}, got {got} ({name})")]
    UnexpectedIndex {
        expected: usize,
        got: usize,
        name: String,
    },
    #[error("Expect vec to have index")]
    ExpectVecToHaveIndex,
    #[error("Failed to convert key to usize")]
    FailedKeyTryIntoUsize,
    #[error("Different format received: {received:?}, expected: {expected:?}")]
    DifferentFormat { received: Format, expected: Format },
    #[error("Version cannot be zero, can't verify endianness otherwise")]
    VersionCannotBeZero,
    #[error("Stamp mismatch: file stamp {file:?} != vec stamp {vec:?}")]
    StampMismatch { file: Stamp, vec: Stamp },
    #[error("Corrupted region: invalid length {region_len}")]
    CorruptedRegion { name: String, region_len: usize },
    #[error("Decompression mismatch: expected {expected_len} values, got {actual_len}")]
    DecompressionMismatch {
        expected_len: usize,
        actual_len: usize,
    },
    #[error("Cannot remove vec: pages still referenced")]
    PagesStillReferenced,
    #[error("Invalid format byte: {0}")]
    InvalidFormat(u8),
    #[error("Invalid argument: {0}")]
    InvalidArgument(&'static str),
    #[error("Arithmetic overflow")]
    Overflow,
    #[error("Arithmetic underflow")]
    Underflow,
}

impl Error {
    /// Returns true if this error is due to a file lock (another process has the database open).
    /// Lock errors are transient and should not trigger data deletion.
    pub fn is_lock_error(&self) -> bool {
        matches!(self, Error::TryLockError(_))
    }

    /// Returns true if this error indicates data corruption or version incompatibility.
    /// These errors may require resetting/deleting the data to recover.
    pub fn is_data_error(&self) -> bool {
        match self {
            Error::IO(io_err) => is_io_data_error(io_err),
            Error::RawDB(rawdb::Error::IO(io_err)) => is_io_data_error(io_err),
            Error::RawDB(rawdb::Error::CorruptedMetadata(_)) => true,
            Error::RawDB(rawdb::Error::InvalidMetadataSize { .. }) => true,
            Error::RawDB(rawdb::Error::EmptyMetadata) => true,
            Error::DifferentVersion { .. }
            | Error::DifferentFormat { .. }
            | Error::StampMismatch { .. }
            | Error::CorruptedRegion { .. }
            | Error::DecompressionMismatch { .. }
            | Error::WrongEndian
            | Error::WrongLength { .. }
            | Error::InvalidFormat(_) => true,
            _ => false,
        }
    }
}

fn is_io_data_error(io_err: &io::Error) -> bool {
    matches!(
        io_err.kind(),
        io::ErrorKind::IsADirectory | io::ErrorKind::NotADirectory
    )
}
