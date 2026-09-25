// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    error::Error as ErrorError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    result::Result as StdResult,
    str::Utf8Error,
};

use log::error;
use sfa::Error as SfaError;

use crate::CompressionType;

/// Result using the LSM tree's error type.
pub type Result<T> = StdResult<T, Error>;

/// Represents errors that can occur in the LSM-tree
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// I/O error
    Io(IoError),

    /// Decompression failed
    Decompress(CompressionType),

    /// Invalid or unparsable data format version
    InvalidVersion(u8),

    /// Some required files could not be recovered from disk
    Unrecoverable,

    /// Invalid enum tag
    InvalidTag((&'static str, u8)),

    /// A record does not match the requested fixed width.
    InvalidRecordLength {
        /// Requested width in bytes.
        expected: usize,
        /// Width found in the block.
        actual: usize,
    },

    /// Invalid block trailer
    InvalidTrailer,

    /// Invalid block header
    InvalidHeader(&'static str),

    /// UTF-8 error
    Utf8(Utf8Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "LsmTreeError: {self:?}")
    }
}

impl ErrorError for Error {
    fn source(&self) -> Option<&(dyn ErrorError + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<SfaError> for Error {
    fn from(value: SfaError) -> Self {
        match value {
            SfaError::Io(e) => Self::from(e),
            SfaError::ChecksumMismatch { .. } => {
                error!("Archive ToC checksum mismatch");
                Self::Unrecoverable
            }
            SfaError::InvalidHeader => {
                error!("Invalid archive header");
                Self::Unrecoverable
            }
            SfaError::InvalidVersion => {
                error!("Invalid archive version");
                Self::Unrecoverable
            }
            SfaError::UnsupportedChecksumType => {
                error!("Invalid archive checksum type");
                Self::Unrecoverable
            }
        }
    }
}

impl From<IoError> for Error {
    fn from(value: IoError) -> Self {
        Self::Io(value)
    }
}
