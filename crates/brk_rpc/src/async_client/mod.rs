use std::sync::Arc;

use brk_error::Result;
use brk_types::{Height, Txid};

use crate::{Auth, rpc_client::ClientInner};

use inner::Inner;

mod connection;
mod inner;

/// Async RPC transport. Clones share its connection and credentials.
/// Callers apply their request deadline.
#[derive(Clone)]
pub struct AsyncClient(Arc<Inner>);

impl AsyncClient {
    fn new(url: &str, auth: Auth) -> Result<Self> {
        Ok(Self(Arc::new(Inner::new(url, auth)?)))
    }

    pub async fn get_last_height(&self) -> Result<Height> {
        self.0.get_last_height().await
    }

    /// Submit without retrying an ambiguous transport failure. Cancelling the
    /// future drops queued work or closes its active connection, but cannot undo
    /// an already received submission. An error may leave the outcome unknown.
    pub async fn send_raw_transaction(&self, hex: &str) -> Result<Txid> {
        self.0.send_raw_transaction(hex).await
    }
}

impl ClientInner {
    pub fn asynchronous(&self) -> Result<AsyncClient> {
        AsyncClient::new(&self.url, self.auth.clone())
    }
}
