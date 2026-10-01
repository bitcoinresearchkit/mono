#![doc = include_str!("../README.md")]

#[cfg(feature = "bitcoin")]
use bitcoin::address::FromScriptError;
#[cfg(feature = "bitcoin")]
use bitcoin::block::Bip34Error;
#[cfg(feature = "bitcoin")]
use bitcoin::consensus::encode::Error as EncodeError;
#[cfg(feature = "bitcoin")]
use bitcoin::consensus::encode::FromHexError;
#[cfg(feature = "bitcoin")]
use bitcoin::hex::HexToArrayError;
#[cfg(feature = "corepc")]
use corepc_jsonrpc::error::Error as ErrorError;
#[cfg(feature = "fjall")]
use fjall::Error as FjallError;
#[cfg(feature = "jiff")]
use jiff::Error as JiffError;
#[cfg(feature = "pco")]
use pco::errors::PcoError;
#[cfg(feature = "serde_json")]
use serde_json::Error as SerdeJsonError;
use std::{
    borrow::Cow,
    fmt,
    io::{self, Error as IoError},
    path::PathBuf,
    result::Result as StdResult,
    time,
};

use thiserror::Error;

#[cfg(feature = "tokio")]
use tokio::task::JoinError;
#[cfg(feature = "ureq")]
use ureq::Error as UreqError;
#[cfg(feature = "vecdb")]
use vecdb::Error as VecdbError;
#[cfg(feature = "vecdb")]
use vecdb::RawDBError;

/// Result using BRK's shared error type.
pub type Result<T> = StdResult<T, Error>;

/// Convert `Option<T>` into a result without panicking.
///
/// Replaces `.unwrap()` in query paths so a missing value returns
/// HTTP 500 instead of crashing the server (`panic = "abort"`).
pub trait OptionData<T> {
    fn data(self) -> Result<T>;
}

impl<T> OptionData<T> for Option<T> {
    #[inline]
    fn data(self) -> Result<T> {
        self.ok_or(Error::Internal("data unavailable"))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[cfg(feature = "corepc")]
    #[error(transparent)]
    CorepcRPC(#[from] ErrorError),

    #[cfg(feature = "jiff")]
    #[error(transparent)]
    Jiff(#[from] JiffError),

    #[cfg(feature = "fjall")]
    #[error(transparent)]
    Fjall(#[from] FjallError),

    #[cfg(feature = "vecdb")]
    #[error(transparent)]
    VecDB(#[from] VecdbError),

    #[cfg(feature = "vecdb")]
    #[error(transparent)]
    RawDB(#[from] RawDBError),

    #[cfg(feature = "ureq")]
    #[error(transparent)]
    Ureq(#[from] UreqError),

    #[error(transparent)]
    SystemTimeError(#[from] time::SystemTimeError),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinConsensusEncode(#[from] EncodeError),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinBip34Error(#[from] Bip34Error),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinHexError(#[from] FromHexError),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinFromScriptError(#[from] FromScriptError),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinHexToArrayError(#[from] HexToArrayError),

    #[cfg(feature = "pco")]
    #[error(transparent)]
    Pco(#[from] PcoError),

    #[cfg(feature = "serde_json")]
    #[error(transparent)]
    SerdeJSON(#[from] SerdeJsonError),

    #[cfg(feature = "tokio")]
    #[error(transparent)]
    TokioJoin(#[from] JoinError),

    #[error("ZeroCopy error")]
    ZeroCopyError,

    #[error("Wrong length, expected: {expected}, received: {received}")]
    WrongLength { expected: usize, received: usize },

    #[error("Wrong address type")]
    WrongAddrType,

    #[error("Date is outside the supported index range")]
    UnindexableDate,

    #[error("Quick cache error")]
    QuickCacheError,

    #[error("The provided address appears to be invalid")]
    InvalidAddr,

    #[error("Invalid network")]
    InvalidNetwork,

    #[error("The provided TXID appears to be invalid")]
    InvalidTxid,

    #[error("Mempool data is not available")]
    MempoolNotAvailable,

    #[error("State is updating")]
    StateUpdating,

    #[error("Read timed out waiting for published data")]
    ReadTimeout,

    #[error("Address not found in the blockchain (no transaction history)")]
    UnknownAddr,

    #[error("Failed to find the TXID in the blockchain")]
    UnknownTxid,

    #[error("Unsupported type ({0})")]
    UnsupportedType(String),

    // Generic errors with context
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    OutOfRange(Cow<'static, str>),

    #[error("{0}")]
    Parse(String),

    #[error("Internal error: {0}")]
    Internal(&'static str),

    #[error("Authentication failed")]
    AuthFailed,

    // Series-specific errors
    #[error("{0}")]
    SeriesNotFound(SeriesNotFound),

    #[error("'{series}' doesn't support the requested index. Try: {supported}")]
    SeriesUnsupportedIndex { series: String, supported: String },

    #[error("No series specified")]
    NoSeries,

    #[error("No data available")]
    NoData,

    #[error("Request weight {requested} exceeds maximum {max}")]
    WeightExceeded { requested: usize, max: usize },

    #[error("Too many unspent transaction outputs (>1000).")]
    TooManyUtxos,

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Fetch failed after retries: {0}")]
    FetchFailed(String),

    #[error("HTTP {status}: {url}")]
    HttpStatus { status: u16, url: String },

    #[error("Version mismatch at {path:?}: expected {expected}, found {found}")]
    VersionMismatch {
        path: PathBuf,
        expected: usize,
        found: usize,
    },
}

impl Error {
    /// Returns true if this error is due to a file lock (another process has the database open).
    /// Lock errors are transient and should not trigger data deletion.
    #[cfg(feature = "vecdb")]
    pub fn is_lock_error(&self) -> bool {
        let is_vecdb_lock = matches!(self, Error::VecDB(e) if e.is_lock_error())
            || matches!(self, Error::RawDB(RawDBError::TryLock(_)));
        #[cfg(feature = "fjall")]
        {
            is_vecdb_lock || matches!(self, Error::Fjall(FjallError::Locked))
        }
        #[cfg(not(feature = "fjall"))]
        {
            is_vecdb_lock
        }
    }

    /// Returns true if this error indicates data corruption or version incompatibility.
    /// These errors may require resetting/deleting the data to recover.
    #[cfg(feature = "vecdb")]
    pub fn is_data_error(&self) -> bool {
        let is_vecdb_data = matches!(self, Error::VecDB(e) if e.is_data_error())
            || matches!(self, Error::VersionMismatch { .. });
        #[cfg(feature = "fjall")]
        {
            is_vecdb_data || matches!(self, Error::Fjall(error) if error.is_data_error())
        }
        #[cfg(not(feature = "fjall"))]
        {
            is_vecdb_data
        }
    }

    /// Returns true if this network/fetch error indicates a permanent/blocking condition
    /// that won't be resolved by retrying (e.g., DNS failure, connection refused, blocked endpoint).
    /// Returns false for transient errors worth retrying (timeouts, rate limits, server errors).
    pub fn is_network_permanently_blocked(&self) -> bool {
        match self {
            #[cfg(feature = "ureq")]
            Error::Ureq(e) => is_ureq_error_permanent(e),
            Error::IO(e) => is_io_error_permanent(e),
            // 403 Forbidden suggests IP/geo blocking; 429 and 5xx are transient
            Error::HttpStatus { status, .. } => *status == 403,
            // Other errors are data/parsing related, not network - treat as transient
            _ => false,
        }
    }
}

#[cfg(all(test, feature = "vecdb"))]
#[path = "../tests/unit/lib.rs"]
mod tests;

#[cfg(feature = "ureq")]
fn is_ureq_error_permanent(e: &UreqError) -> bool {
    let msg = format!("{:?}", e);
    msg.contains("nodename nor servname")
        || msg.contains("Name or service not known")
        || msg.contains("No such host")
        || msg.contains("connection refused")
        || msg.contains("Connection refused")
        || msg.contains("certificate")
        || msg.contains("SSL")
        || msg.contains("TLS")
        || msg.contains("handshake")
}

fn is_io_error_permanent(e: &IoError) -> bool {
    use std::io::ErrorKind::*;
    match e.kind() {
        // Permanent errors
        ConnectionRefused | PermissionDenied | AddrNotAvailable => true,
        // Check the error message for DNS failures
        _ => {
            let msg = e.to_string();
            msg.contains("nodename nor servname")
                || msg.contains("Name or service not known")
                || msg.contains("No such host")
        }
    }
}

/// Maximum length of a user-supplied series name in error messages before
/// truncating with an ellipsis.
const SERIES_NAME_MAX_DISPLAY_LEN: usize = 100;

/// Truncate a user-supplied series name for inclusion in an error message,
/// appending an ellipsis if it exceeds the display cap. Used for both
/// `SeriesNotFound` and `SeriesUnsupportedIndex` so far-too-long names don't
/// blow up the response body.
pub fn truncate_series_name(mut series: String) -> String {
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

impl SeriesNotFound {
    pub fn new(series: String, suggestions: Vec<&'static str>, total_matches: usize) -> Self {
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
