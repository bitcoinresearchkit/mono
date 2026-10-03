use std::{sync::Arc, time::Duration};

#[cfg(feature = "async")]
use brk_error::Error;
use brk_error::Result;
#[cfg(feature = "async")]
use corepc_jsonrpc::error::Error as ErrorError;

use crate::Auth;

#[cfg(feature = "async")]
use crate::AsyncClient;

mod block_template;
mod inner;
mod mempool_entry;
mod mempool_state;
mod methods;
mod rpc_call;
mod txid_array_parser;

pub use inner::ClientInner;
pub use mempool_state::MempoolState;

/// Explicit Core decode/policy rejections are invalid input, unlike an
/// infrastructure error whose submission outcome may be unknown.
#[cfg(feature = "async")]
pub fn transaction_error(error: Error) -> Error {
    if let Error::CorepcRPC(ErrorError::Rpc(rpc)) = &error
        && matches!(rpc.code, -22 | -25 | -26 | -27)
    {
        return Error::Parse(rpc.message.clone());
    }
    error
}

/// Bitcoin Core RPC client. Thread-safe and cheap to clone.
#[derive(Debug, Clone)]
pub struct Client(Arc<ClientInner>);

impl Client {
    #[cfg(feature = "async")]
    pub fn asynchronous(&self) -> Result<AsyncClient> {
        self.0.asynchronous()
    }
    pub fn new(url: &str, auth: Auth) -> Result<Self> {
        Ok(Self(Arc::new(ClientInner::new(
            url,
            auth,
            1_000_000,
            Duration::from_secs(1),
        )?)))
    }
}
