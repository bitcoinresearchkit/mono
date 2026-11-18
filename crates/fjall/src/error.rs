use std::{
    error::Error as ErrorError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    result::Result as StdResult,
};

use lsm_tree::Error as LsmTreeError;

/// Result using Fjall's error type.
pub type Result<T> = StdResult<T, Error>;

/// Errors returned by BRK's table-only database.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Error in the underlying LSM tree.
    Storage(LsmTreeError),
    /// Filesystem error.
    Io(IoError),
    /// Invalid or unsupported database format.
    InvalidVersion,
    /// Another process holds the database lock.
    Locked,
}

impl Error {
    /// Returns whether reopening requires rebuilding the derived database: only for data that is
    /// ours and fails a check. An I/O error, the lock, or a variant the tree adds later stops the
    /// caller without deleting anything.
    #[must_use]
    pub fn is_data_error(&self) -> bool {
        match self {
            Self::InvalidVersion => true,
            Self::Storage(error) => matches!(
                error,
                LsmTreeError::Decompress(_)
                    | LsmTreeError::InvalidVersion(_)
                    | LsmTreeError::Unrecoverable(_)
                    | LsmTreeError::InvalidTag(_)
                    | LsmTreeError::InvalidRecordLength { .. }
                    | LsmTreeError::InvalidTrailer
                    | LsmTreeError::InvalidHeader(_)
                    | LsmTreeError::Utf8(_)
            ),
            Self::Io(_) | Self::Locked => false,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "FjallError: {self:?}")
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}

impl From<LsmTreeError> for Error {
    fn from(error: LsmTreeError) -> Self {
        Self::Storage(error)
    }
}

impl ErrorError for Error {
    fn source(&self) -> Option<&(dyn ErrorError + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::InvalidVersion | Self::Locked => None,
        }
    }
}
