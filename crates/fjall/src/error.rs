/// Result using Fjall's error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by BRK's table-only database.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Error in the underlying LSM tree.
    Storage(lsm_tree::Error),
    /// Filesystem error.
    Io(std::io::Error),
    /// Invalid or unsupported database format.
    InvalidVersion,
    /// Another process holds the database lock.
    Locked,
}

impl Error {
    /// Returns whether reopening requires rebuilding the derived database.
    #[must_use]
    pub fn is_data_error(&self) -> bool {
        match self {
            Self::InvalidVersion => true,
            Self::Storage(error) => !matches!(error, lsm_tree::Error::Io(_)),
            Self::Io(_) | Self::Locked => false,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FjallError: {self:?}")
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<lsm_tree::Error> for Error {
    fn from(error: lsm_tree::Error) -> Self {
        Self::Storage(error)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::InvalidVersion | Self::Locked => None,
        }
    }
}
