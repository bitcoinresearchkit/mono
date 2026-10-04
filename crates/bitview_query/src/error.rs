use std::{borrow::Cow, fmt, result::Result as StdResult};

use thiserror::Error as ThisError;

/// Result of a query.
pub type Result<T> = StdResult<T, Error>;

/// What a query can answer besides data: the API's error vocabulary.
///
/// Failures from lower layers (storage, RPC, parsing of stored data) arrive as
/// [`Error::Lower`] and are always internal. The few conditions those layers name on
/// purpose keep their meaning through `From<brk_error::Error>`: an updating mempool,
/// invalid address input and a date outside the indexes.
#[derive(Debug, ThisError)]
pub enum Error {
    // The request cannot be answered as asked.
    #[error("The provided address appears to be invalid")]
    InvalidAddr,
    #[error("Invalid network")]
    InvalidNetwork,
    #[error("{0}")]
    InvalidParam(String),
    #[error("Unsupported type ({0})")]
    UnsupportedType(String),
    #[error("No series specified")]
    NoSeries,
    #[error("'{series}' doesn't support the requested index. Try: {supported}")]
    SeriesUnsupportedIndex { series: String, supported: String },
    #[error("Request weight {requested} exceeds maximum {max}")]
    WeightExceeded { requested: usize, max: usize },
    #[error("Too many unspent transaction outputs (more than {max})")]
    TooManyUtxos { max: usize },

    // Nothing to answer with.
    #[error("Address not found in the blockchain (no transaction history)")]
    UnknownAddr,
    #[error("Failed to find the TXID in the blockchain")]
    UnknownTxid,
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    OutOfRange(Cow<'static, str>),
    #[error("Date is outside the supported index range")]
    UnindexableDate,
    #[error("No data available")]
    NoData,
    #[error("{0}")]
    SeriesNotFound(SeriesNotFound),

    // Try again later.
    #[error("Mempool data is not available")]
    MempoolNotAvailable,
    #[error("State is updating")]
    StateUpdating,
    #[error("Read timed out waiting for published data")]
    ReadTimeout,

    // Failures.
    #[error("Internal error: {0}")]
    Internal(&'static str),
    #[error(transparent)]
    Lower(brk_error::Error),
}

impl Error {
    /// Another process holds the database open: transient, never a reason to delete data.
    pub fn is_lock_error(&self) -> bool {
        matches!(self, Self::Lower(error) if error.is_lock_error())
    }
}

impl From<brk_error::Error> for Error {
    fn from(error: brk_error::Error) -> Self {
        use brk_error::Error as Lower;
        match error {
            Lower::StateUpdating => Self::StateUpdating,
            Lower::InvalidAddr => Self::InvalidAddr,
            Lower::InvalidNetwork => Self::InvalidNetwork,
            Lower::UnindexableDate => Self::UnindexableDate,
            error => Self::Lower(error),
        }
    }
}

/// Turns a missing value into an internal error instead of a panic.
pub trait OptionData<T> {
    fn data(self) -> Result<T>;
}

impl<T> OptionData<T> for Option<T> {
    #[inline]
    fn data(self) -> Result<T> {
        self.ok_or(Error::Internal("data unavailable"))
    }
}

/// Lower-layer error types the query propagates with `?`; all are internal failures.
macro_rules! lower {
    ($($ty:ty),* $(,)?) => {$(
        impl From<$ty> for Error {
            fn from(error: $ty) -> Self {
                Self::Lower(error.into())
            }
        }
    )*};
}

lower!(
    std::io::Error,
    vecdb::Error,
    vecdb::RawDBError,
    jiff::Error,
    serde_json::Error,
);
lower!(tokio::task::JoinError);

/// Maximum length of a user-supplied series name in error messages before
/// truncating with an ellipsis.
#[cfg(feature = "series")]
const SERIES_NAME_MAX_DISPLAY_LEN: usize = 100;

/// Truncate a user-supplied series name for inclusion in an error message,
/// appending an ellipsis if it exceeds the display cap. Used for both
/// `SeriesNotFound` and `SeriesUnsupportedIndex` so far-too-long names don't
/// blow up the response body.
#[cfg(feature = "series")]
pub(crate) fn truncate_series_name(mut series: String) -> String {
    if series.len() > SERIES_NAME_MAX_DISPLAY_LEN {
        series.truncate(series.floor_char_boundary(SERIES_NAME_MAX_DISPLAY_LEN));
        series.push_str("...");
    }
    series
}

#[derive(Debug)]
pub struct SeriesNotFound {
    series: String,
    suggestions: Vec<&'static str>,
    total_matches: usize,
}

#[cfg(feature = "series")]
impl SeriesNotFound {
    pub(crate) fn new(
        series: String,
        suggestions: Vec<&'static str>,
        total_matches: usize,
    ) -> Self {
        Self {
            series: truncate_series_name(series),
            suggestions,
            total_matches,
        }
    }
}

impl fmt::Display for SeriesNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}' not found", self.series)?;

        if self.suggestions.is_empty() {
            return Ok(());
        }

        f.write_str(", did you mean ")?;
        for (index, suggestion) in self.suggestions.iter().enumerate() {
            if index != 0 {
                f.write_str(", ")?;
            }
            write!(f, "'{suggestion}'")?;
        }
        f.write_str("?")?;

        let remaining = self.total_matches.saturating_sub(self.suggestions.len());
        if remaining > 0 {
            write!(
                f,
                " ({remaining} more — /api/series/search?q={} for all)",
                self.series
            )?;
        }

        Ok(())
    }
}
