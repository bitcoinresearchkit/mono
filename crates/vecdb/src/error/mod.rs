#[cfg(feature = "lz4")]
use lz4_flex::block::DecompressError;
#[cfg(feature = "pco")]
use pco::errors::PcoError;
use std::{io, result::Result as StdResult};

use rawdb::Error as RawdbError;
use thiserror::Error;

use crate::{Format, Stamp, Version};

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Write failed; discard this vector and validate or rebuild its data before continuing")]
    WriteFailed,
    #[error(transparent)]
    IO(#[from] io::Error),
    #[cfg(feature = "zerocopy")]
    #[error("Zero-copy conversion failed")]
    ZeroCopyError,
    #[cfg(feature = "pco")]
    #[error(transparent)]
    PCO(#[from] PcoError),
    #[cfg(feature = "lz4")]
    #[error(transparent)]
    LZ4(#[from] DecompressError),
    #[error(transparent)]
    RawDB(#[from] RawdbError),
    #[error("Length mismatch: received {received}, expected {expected}")]
    WrongLength { received: usize, expected: usize },
    #[error("Iterator ended")]
    IteratorEnded,
    #[error("Version mismatch: received {received:?}, expected {expected:?}")]
    DifferentVersion {
        received: Version,
        expected: Version,
    },
    #[error("Index {index} out of bounds for {name} (len {len})")]
    IndexTooHigh {
        index: usize,
        len: usize,
        name: String,
    },
    #[error("Missing compressed page")]
    MissingPage,
    #[error("Format mismatch: received {received:?}, expected {expected:?}")]
    DifferentFormat { received: Format, expected: Format },
    #[error("Value size mismatch: received {received} bytes, expected {expected} bytes")]
    DifferentValueSize { received: u16, expected: u16 },
    #[error("Stamp mismatch: file stamp {file:?} != vec stamp {vec:?}")]
    StampMismatch { file: Stamp, vec: Stamp },
    #[error("Corrupted region {name}: invalid length {region_len}")]
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
    /// The stored vector has another schema (version, format, value size or a length it cannot
    /// hold): import deletes it and starts over.
    pub(crate) fn is_schema_mismatch(&self) -> bool {
        matches!(
            self,
            Error::WrongLength { .. }
                | Error::DifferentFormat { .. }
                | Error::DifferentValueSize { .. }
                | Error::DifferentVersion { .. }
        )
    }

    /// Returns true if this error is due to a file lock (another process has the database open).
    /// Lock errors are transient and should not trigger data deletion.
    pub fn is_lock_error(&self) -> bool {
        matches!(self, Error::RawDB(RawdbError::TryLock(_)))
    }

    /// Returns true if this error indicates data corruption or version incompatibility.
    /// These errors may require resetting/deleting the data to recover.
    pub fn is_data_error(&self) -> bool {
        match self {
            Error::IO(io_err) => is_io_data_error(io_err),
            Error::RawDB(RawdbError::IO(io_err)) => is_io_data_error(io_err),
            Error::RawDB(RawdbError::CorruptedMetadata(_)) => true,
            Error::DifferentVersion { .. }
            | Error::DifferentFormat { .. }
            | Error::DifferentValueSize { .. }
            | Error::StampMismatch { .. }
            | Error::CorruptedRegion { .. }
            | Error::DecompressionMismatch { .. }
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
