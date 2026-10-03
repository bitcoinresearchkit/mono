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
use std::{borrow::Cow, io, path::PathBuf, result::Result as StdResult, time};

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

    #[error("Wrong length, expected: {expected}, received: {received}")]
    WrongLength { expected: usize, received: usize },

    #[error("Wrong address type")]
    WrongAddrType,

    #[error("Date is outside the supported index range")]
    UnindexableDate,

    #[error("The provided address appears to be invalid")]
    InvalidAddr,

    #[error("Invalid network")]
    InvalidNetwork,

    #[error("State is updating")]
    StateUpdating,

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

    #[error(
        "No RPC credentials: no rpcpassword is set and there's no cookie file at {cookie:?} (is bitcoind running?)"
    )]
    NoRpcCredentials { cookie: PathBuf },

    /// The node refused a submitted transaction (policy or consensus); the reason is the node's.
    #[error("{0}")]
    TxRejected(String),

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
}
