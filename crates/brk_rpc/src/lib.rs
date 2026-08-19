use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use brk_types::{Sats, Txid, Weight};

mod client;
mod methods;

use client::ClientInner;
pub use corepc_types::v17::{GetBlockHeaderVerbose, GetBlockVerboseOne, GetTxOut};
pub use methods::MempoolState;

/// One transaction from `getblocktemplate`. Carries the full decoded
/// body and stats so block 0 can be projected without a follow-up
/// `getmempoolentry`/`getrawtransaction` per tx; that follow-up was the
/// source of the GBT/listing race that used to skip cycles.
#[derive(Debug, Clone)]
pub struct BlockTemplateTx {
    pub txid: Txid,
    pub fee: Sats,
    pub weight: Weight,
    /// Parent txids also in this template (Core's own ancestor
    /// accounting, resolved from the wire-level 1-based indices).
    pub depends: Vec<Txid>,
    pub tx: bitcoin::Transaction,
}

#[derive(Clone, Debug)]
pub enum Auth {
    None,
    UserPass(String, String),
    CookieFile(PathBuf),
}

/// Bitcoin Core RPC client. Thread-safe and cheap to clone.
#[derive(Debug, Clone)]
pub struct Client(pub(crate) Arc<ClientInner>);

impl Client {
    pub fn new(url: &str, auth: Auth) -> brk_error::Result<Self> {
        Self::new_with(url, auth, 1_000_000, Duration::from_secs(1))
    }

    pub fn new_with(
        url: &str,
        auth: Auth,
        max_retries: usize,
        retry_delay: Duration,
    ) -> brk_error::Result<Self> {
        Ok(Self(Arc::new(ClientInner::new(
            url,
            auth,
            max_retries,
            retry_delay,
        )?)))
    }

    pub fn default_url() -> &'static str {
        "http://localhost:8332"
    }

    pub fn default_bitcoin_path() -> PathBuf {
        if env::consts::OS == "macos" {
            Self::default_mac_bitcoin_path()
        } else {
            Self::default_linux_bitcoin_path()
        }
    }

    pub fn default_linux_bitcoin_path() -> PathBuf {
        Path::new(&env::var("HOME").unwrap()).join(".bitcoin")
    }

    pub fn default_mac_bitcoin_path() -> PathBuf {
        Path::new(&env::var("HOME").unwrap())
            .join("Library")
            .join("Application Support")
            .join("Bitcoin")
    }
}
