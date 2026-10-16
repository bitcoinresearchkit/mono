#![doc = include_str!("../README.md")]

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
#[cfg(feature = "serde_json")]
use serde_json::Error as SerdeJsonError;
use std::{borrow::Cow, io, path::PathBuf, result::Result as StdResult};

use thiserror::Error;

#[cfg(feature = "tokio")]
use tokio::task::JoinError;
#[cfg(feature = "vecdb")]
use vecdb::Error as VecdbError;
#[cfg(feature = "vecdb")]
use vecdb::RawDBError;

/// Result using BRK's shared error type.
pub type Result<T> = StdResult<T, Error>;

/// Convert `Option<T>` into a result without panicking.
///
/// Replaces `.unwrap()` in plugin code so a missing value fails the computation with an
/// error instead of aborting the process (`panic = "abort"`).
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

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinConsensusEncode(#[from] EncodeError),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinHexError(#[from] FromHexError),

    #[cfg(feature = "bitcoin")]
    #[error(transparent)]
    BitcoinHexToArrayError(#[from] HexToArrayError),

    #[cfg(feature = "serde_json")]
    #[error(transparent)]
    SerdeJSON(#[from] SerdeJsonError),

    #[cfg(feature = "tokio")]
    #[error(transparent)]
    TokioJoin(#[from] JoinError),

    #[error("Wrong length, expected: {expected}, received: {received}")]
    WrongLength { expected: usize, received: usize },

    #[error("Wrong address type")]
    WrongAddrType,

    #[error("Date is outside the supported index range")]
    UnindexableDate,

    #[error("The provided address appears to be invalid")]
    InvalidAddress,

    #[error("Invalid network")]
    InvalidNetwork,

    #[error("State is updating")]
    StateUpdating,

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

    #[error(
        "No RPC credentials: no rpcpassword is set and there's no cookie file at {cookie:?} (is bitcoind running?)"
    )]
    NoRpcCredentials { cookie: PathBuf },

    #[error(
        "Bitcoin Core rejected the RPC credentials (check rpcuser and rpcpassword, or the cookie file)"
    )]
    RpcAuthFailed,

    #[error(
        "Bitcoin Core refused this RPC request (check rpcallowip, or rpcwhitelist for this user)"
    )]
    RpcForbidden,

    /// The node refused a submitted transaction (policy or consensus); the reason is the node's.
    #[error("{0}")]
    TxRejected(String),

    #[error("Data without a version marker at {path:?}")]
    MissingVersion { path: PathBuf },

    #[error("Version mismatch at {path:?}: expected {expected}, found {found}")]
    VersionMismatch {
        path: PathBuf,
        expected: usize,
        found: usize,
    },
}

/// Raw database errors go through vecdb's, which owns their lock and corruption classification.
#[cfg(feature = "vecdb")]
impl From<RawDBError> for Error {
    #[inline]
    fn from(error: RawDBError) -> Self {
        Self::VecDB(error.into())
    }
}

impl Error {
    /// Returns true if this error is due to a file lock (another process has the database open).
    /// Lock errors are transient and should not trigger data deletion.
    pub fn is_lock_error(&self) -> bool {
        match self {
            #[cfg(feature = "vecdb")]
            Error::VecDB(e) => e.is_lock_error(),
            #[cfg(feature = "fjall")]
            Error::Fjall(FjallError::Locked) => true,
            _ => false,
        }
    }

    /// Returns true if this error indicates data corruption or version incompatibility.
    /// These errors may require resetting/deleting the data to recover.
    pub fn is_data_error(&self) -> bool {
        match self {
            #[cfg(feature = "vecdb")]
            Error::VecDB(e) => e.is_data_error(),
            #[cfg(feature = "fjall")]
            Error::Fjall(e) => e.is_data_error(),
            Error::VersionMismatch { .. } | Error::MissingVersion { .. } => true,
            _ => false,
        }
    }
}
