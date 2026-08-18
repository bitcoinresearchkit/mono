#![doc = include_str!("../README.md")]

use std::{borrow::Cow, fmt, io, path::PathBuf, result, time};

use thiserror::Error;

pub type Result<T, E = Error> = result::Result<T, E>;

/// Convert `Option<T>` → `Result<T>` without panicking.
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
    CorepcRPC(#[from] corepc_jsonrpc::error::Error),

    #[cfg(feature = "jiff")]
    #[error(transparent)]
    Jiff(#[from] jiff::Error),

    #[cfg(feature = "fjall")]
    #[error(transparent)]
    Fjall(#[from] fjall::Error),

    #[cfg(feature = "vecdb")]
    #[error(transparent)]
    VecDB(#[from] vecdb::Error),

    #[cfg(feature = "vecdb")]
    #[error(transparent)]
    RawDB(#[from] vecdb::RawDBError),

    #[cfg(feature = "ureq")]
    #[error(transparent)]
    Ureq(#[from] ureq::Error),

    #[error(transparent)]
    SystemTimeError(#[from] time::SystemTimeError),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinConsensusEncode(#[from] bitcoin::consensus::encode::Error),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinBip34Error(#[from] bitcoin::block::Bip34Error),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinHexError(#[from] bitcoin::consensus::encode::FromHexError),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinFromScriptError(#[from] bitcoin::address::FromScriptError),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinHexToArrayError(#[from] bitcoin::hex::HexToArrayError),

    #[cfg(feature = "pco")]
    #[error(transparent)]
    Pco(#[from] pco::errors::PcoError),

    #[cfg(feature = "serde_json")]
    #[error(transparent)]
    SerdeJSON(#[from] serde_json::Error),

    #[cfg(feature = "tokio")]
    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    #[error("ZeroCopy error")]
    ZeroCopyError,

    #[error("Wrong length, expected: {expected}, received: {received}")]
    WrongLength { expected: usize, received: usize },

    #[error("Wrong address type")]
    WrongAddrType,

    #[error("Date cannot be indexed, must be 2009-01-03, 2009-01-09 or greater")]
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
        let is_vecdb_lock = matches!(self, Error::VecDB(e) if e.is_lock_error());
        #[cfg(feature = "fjall")]
        {
            is_vecdb_lock || matches!(self, Error::Fjall(fjall::Error::Locked))
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

#[cfg(feature = "ureq")]
fn is_ureq_error_permanent(e: &ureq::Error) -> bool {
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

fn is_io_error_permanent(e: &std::io::Error) -> bool {
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
        series.truncate(SERIES_NAME_MAX_DISPLAY_LEN);
        series.push_str("...");
    }
    series
}

#[derive(Debug)]
pub struct SeriesNotFound {
    pub series: String,
    pub suggestions: Vec<&'static str>,
    pub total_matches: usize,
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

        let quoted: Vec<_> = self.suggestions.iter().map(|s| format!("'{s}'")).collect();
        write!(f, ", did you mean {}?", quoted.join(", "))?;

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
