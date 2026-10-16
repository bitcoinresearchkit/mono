use std::{thread::sleep, time::Duration};

use brk_error::{Error, Result};
use corepc_jsonrpc::{
    Client as JsonRpcClient, Request, Response,
    error::{Error as JsonRpcError, RpcError},
};
use serde::{Deserialize, Serialize};
use serde_json::{
    from_str,
    value::{RawValue, to_raw_value},
};
use tracing::{debug, info, warn};

use super::{
    http::{Http, HttpError},
    rpc_call::RpcCall,
};
use crate::Auth;

/// Connection-loss retries before giving up: effectively wait for the node forever.
const MAX_RETRIES: usize = 1_000_000;
const RETRY_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct ClientInner {
    /// Kept for [`ClientInner::asynchronous`].
    #[cfg(feature = "async")]
    pub url: String,
    #[cfg(feature = "async")]
    pub auth: Auth,
    client: JsonRpcClient,
}

impl ClientInner {
    pub fn new(url: &str, auth: Auth) -> Result<Self> {
        Ok(Self {
            client: JsonRpcClient::with_transport(Http::new(url, &auth)?),
            #[cfg(feature = "async")]
            url: url.to_string(),
            #[cfg(feature = "async")]
            auth,
        })
    }

    pub fn call_with_retry<T, P>(&self, method: &str, args: &P) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        P: Serialize + ?Sized,
    {
        let raw = to_raw_value(args).map_err(Error::from)?;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                debug!(
                    "Trying to reconnect to Bitcoin Core (attempt {}/{})",
                    attempt, MAX_RETRIES
                );
                sleep(RETRY_DELAY);
            }

            match self.client.call::<T>(method, Some(&raw)) {
                Ok(value) => {
                    if attempt > 0 {
                        info!(
                            "Successfully reconnected to Bitcoin Core after {} attempts",
                            attempt
                        );
                    }
                    return Ok(value);
                }
                Err(e) if is_retriable(&e) => {
                    if attempt == 0 {
                        warn!("Lost connection to Bitcoin Core; reconnecting...");
                    }
                }
                Err(e) => return Err(refusal(e)),
            }
        }

        warn!(
            "Could not reconnect to Bitcoin Core after {} attempts",
            MAX_RETRIES + 1
        );
        Err(JsonRpcError::Rpc(RpcError {
            code: -1,
            message: "Max retries exceeded".to_string(),
            data: None,
        })
        .into())
    }

    /// One batch round-trip, responses in request order. Only the transport's own replays (a
    /// changed cookie, a dead pooled connection), no reconnect loop: callers decide batch sizing
    /// and failure semantics.
    fn send_batch<'a>(
        &self,
        label: &str,
        calls: impl IntoIterator<Item = (&'a str, &'a RawValue)>,
    ) -> Result<Vec<Option<Response>>> {
        let requests: Vec<Request> = calls
            .into_iter()
            .map(|(method, params)| self.client.build_request(method, Some(params)))
            .collect();
        self.client
            .send_batch(&requests)
            .map_err(|error| match refusal(error) {
                Error::CorepcRPC(error) => Error::Parse(format!("batch {label} failed: {error}")),
                refused => refused,
            })
    }

    fn serialize_args<P: Serialize>(
        batch_args: impl IntoIterator<Item = P>,
    ) -> Result<Vec<Box<RawValue>>> {
        batch_args
            .into_iter()
            .map(|args| to_raw_value(&args).map_err(Error::from))
            .collect()
    }

    /// Send a batch of calls sharing `method`, one set of args per request.
    /// Any failed item fails the whole batch.
    pub fn call_batch<T, P>(
        &self,
        method: &str,
        batch_args: impl IntoIterator<Item = P>,
    ) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
        P: Serialize,
    {
        let params = Self::serialize_args(batch_args)?;
        self.send_batch(method, params.iter().map(|p| (method, &**p)))?
            .into_iter()
            .map(|response| {
                raw_result(response)
                    .and_then(|raw| Ok(from_str(raw.get())?))
                    .map_err(|e| Error::Parse(format!("batch {method} result: {e}")))
            })
            .collect()
    }

    /// Like `call_batch` but reports each item's raw result independently, so
    /// one bad item doesn't nuke an otherwise-healthy chunk. Per-item failures
    /// preserve the underlying `JsonRpcError` so the caller can pattern-match
    /// on the RPC error code. The outer `Result` still fails if the HTTP
    /// round-trip itself fails.
    pub fn call_batch_raw<P: Serialize>(
        &self,
        method: &str,
        batch_args: impl IntoIterator<Item = P>,
    ) -> Result<Vec<Result<Box<RawValue>>>> {
        let params = Self::serialize_args(batch_args)?;
        Ok(self
            .send_batch(method, params.iter().map(|p| (method, &**p)))?
            .into_iter()
            .map(raw_result)
            .collect())
    }

    /// Mixed-method batch: each `(method, args)` pair becomes one request
    /// in a single round-trip. Each result is independently parsed by the
    /// caller using its own `T`. Outer `Result` fails on transport errors;
    /// inner `Result`s fail on per-item RPC errors.
    pub fn call_mixed_batch(&self, calls: &[RpcCall]) -> Result<Vec<Result<Box<RawValue>>>> {
        Ok(self
            .send_batch(
                "mixed",
                calls.iter().map(|call| (call.method, &*call.params)),
            )?
            .into_iter()
            .map(raw_result)
            .collect())
    }
}

/// Moves one batch item's result out of its response, instead of re-parsing
/// and copying the raw JSON through `Response::result`.
fn raw_result(response: Option<Response>) -> Result<Box<RawValue>> {
    let response = response.ok_or(Error::Internal("Missing response in JSON-RPC batch"))?;
    match response.error {
        Some(error) => Err(JsonRpcError::Rpc(error).into()),
        None => Ok(response.result.unwrap_or_else(|| RawValue::NULL.to_owned())),
    }
}

/// Lost connections, server errors and the node's warm-up answers. Client errors (401, 403 from
/// `rpcallowip` or `rpcwhitelist`, 404 for a wrong path) don't change on their own, except a
/// proxy's timeout and rate limit (408, 429).
fn is_retriable(error: &JsonRpcError) -> bool {
    match error {
        // RPC_IN_WARMUP: the node is still loading.
        JsonRpcError::Rpc(e) => e.code == -28,
        JsonRpcError::Transport(e) => match e.downcast_ref() {
            Some(HttpError::Status(408 | 429)) => true,
            Some(HttpError::Status(code)) => !(400..500).contains(code),
            Some(HttpError::Cookie(_)) => false,
            Some(HttpError::Malformed(_) | HttpError::Io(_)) | None => true,
        },
        _ => false,
    }
}

/// The node refused the request itself (credentials, `rpcallowip`), else the error as is.
fn refusal(error: JsonRpcError) -> Error {
    let JsonRpcError::Transport(boxed) = error else {
        return error.into();
    };
    match boxed.downcast::<HttpError>() {
        Ok(http) => match *http {
            HttpError::Status(401) => Error::RpcAuthFailed,
            HttpError::Status(403) => Error::RpcForbidden,
            HttpError::Cookie(error) => Error::IO(error),
            http => JsonRpcError::Transport(Box::new(http)).into(),
        },
        Err(boxed) => JsonRpcError::Transport(boxed).into(),
    }
}
