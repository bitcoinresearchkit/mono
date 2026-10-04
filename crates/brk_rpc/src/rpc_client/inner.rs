use std::{fs::read_to_string, thread::sleep, time::Duration};

use brk_error::{Error, Result};
use corepc_jsonrpc::{
    Client as JsonRpcClient, Request, Response,
    error::{Error as JsonRpcError, RpcError},
    simple_http,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::value::{RawValue, to_raw_value};
use tracing::{debug, info, warn};

use super::rpc_call::RpcCall;
use crate::{Auth, auth::cookie_error};

#[derive(Debug)]
pub struct ClientInner {
    pub url: String,
    pub auth: Auth,
    /// The client and the cookie it authenticates with (cookie auth only), swapped together.
    client: RwLock<(JsonRpcClient, Option<String>)>,
    max_retries: usize,
    retry_delay: Duration,
}

impl ClientInner {
    pub fn new(url: &str, auth: Auth, max_retries: usize, retry_delay: Duration) -> Result<Self> {
        let (client, cookie) = Self::create_client(url, &auth)?;
        Ok(Self {
            url: url.to_string(),
            auth,
            client: RwLock::new((client, cookie)),
            max_retries,
            retry_delay,
        })
    }

    /// Builds a `jsonrpc::Client` using the `simple_http` transport, which
    /// keeps a single pooled TCP socket with reconnect-on-failure. The
    /// upstream `corepc-client` hard-wires `bitreq_http` (one TCP connect
    /// per request), which collapses under concurrent load.
    fn create_client(url: &str, auth: &Auth) -> Result<(JsonRpcClient, Option<String>)> {
        let builder = simple_http::Builder::new()
            .url(url)
            .map_err(|e| Error::Parse(format!("bad rpc url: {e}")))?
            .timeout(Duration::from_secs(60));
        let (builder, cookie) = match auth {
            Auth::None => (builder, None),
            Auth::UserPass(u, p) => (builder.auth(u.clone(), Some(p.clone())), None),
            Auth::CookieFile(path) => {
                let cookie = read_to_string(path).map_err(|error| cookie_error(path, error))?;
                let cookie = cookie.trim().to_owned();
                (builder.cookie_auth(&cookie), Some(cookie))
            }
        };
        Ok((JsonRpcClient::with_transport(builder.build()), cookie))
    }

    fn recreate(&self) -> Result<()> {
        *self.client.write() = Self::create_client(&self.url, &self.auth)?;
        Ok(())
    }

    /// After `rejected` was refused, whether the node has a new cookie: Bitcoin Core writes it
    /// before serving RPC and answers 503 (retried as a lost connection) before deleting it, so a
    /// rejection with the cookie unchanged, or unreadable, is real. Passwords and missing auth
    /// never change.
    fn cookie_changed(&self, rejected: Option<&str>) -> Result<bool> {
        let Auth::CookieFile(path) = &self.auth else {
            return Ok(false);
        };
        let cookie = read_to_string(path).map_err(|error| cookie_error(path, error))?;
        Ok(rejected != Some(cookie.trim()))
    }

    pub fn call_with_retry<T, P>(&self, method: &str, args: &P) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        P: Serialize + ?Sized,
    {
        let raw = to_raw_value(args).map_err(Error::from)?;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                debug!(
                    "Trying to reconnect to Bitcoin Core (attempt {}/{})",
                    attempt, self.max_retries
                );
                self.recreate().ok();
                sleep(self.retry_delay);
            }

            let (result, rejected_cookie) = {
                let client = self.client.read();
                let result = client.0.call::<T>(method, Some(&raw));
                let rejected = match &result {
                    Err(e) if is_auth_rejection(e) => client.1.clone(),
                    _ => None,
                };
                (result, rejected)
            };
            match result {
                Ok(value) => {
                    if attempt > 0 {
                        info!(
                            "Successfully reconnected to Bitcoin Core after {} attempts",
                            attempt
                        );
                    }
                    return Ok(value);
                }
                Err(e) if is_auth_rejection(&e) => {
                    if !self.cookie_changed(rejected_cookie.as_deref())? {
                        return Err(Error::RpcAuthFailed);
                    }
                    warn!("Bitcoin Core rejected the RPC cookie; waiting for its new cookie...");
                }
                Err(e) if is_retriable(&e) => {
                    if attempt == 0 {
                        warn!("Lost connection to Bitcoin Core; reconnecting...");
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        warn!(
            "Could not reconnect to Bitcoin Core after {} attempts",
            self.max_retries + 1
        );
        Err(JsonRpcError::Rpc(RpcError {
            code: -1,
            message: "Max retries exceeded".to_string(),
            data: None,
        })
        .into())
    }

    fn send_batch<P>(
        &self,
        method: &str,
        batch_args: impl IntoIterator<Item = P>,
    ) -> Result<Vec<Option<Response>>>
    where
        P: Serialize,
    {
        let params: Vec<Box<RawValue>> = batch_args
            .into_iter()
            .map(|args| to_raw_value(&args).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;

        let client = &self.client.read().0;
        let requests: Vec<Request> = params
            .iter()
            .map(|params| client.build_request(method, Some(params)))
            .collect();

        client
            .send_batch(&requests)
            .map_err(|error| match is_auth_rejection(&error) {
                true => Error::RpcAuthFailed,
                false => Error::Parse(format!("batch {method} failed: {error}")),
            })
    }

    /// Send a batch of calls sharing `method`, one set of args per request.
    /// No retry: the caller decides batch sizing and failure semantics.
    pub fn call_batch<T, P>(
        &self,
        method: &str,
        batch_args: impl IntoIterator<Item = P>,
    ) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
        P: Serialize,
    {
        self.send_batch(method, batch_args)?
            .into_iter()
            .map(|resp| {
                let resp = resp.ok_or(Error::Internal("Missing response in JSON-RPC batch"))?;
                resp.result::<T>()
                    .map_err(|e| Error::Parse(format!("batch {method} result: {e}")))
            })
            .collect()
    }

    /// Like `call_batch` but reports per-request success/failure independently,
    /// so one bad item doesn't nuke an otherwise-healthy chunk. Per-item
    /// failures preserve the underlying `JsonRpcError` so the caller can
    /// pattern-match on the RPC error code. The outer `Result` still fails
    /// if the HTTP round-trip itself fails.
    pub fn call_batch_per_item<T, P>(
        &self,
        method: &str,
        batch_args: impl IntoIterator<Item = P>,
    ) -> Result<Vec<Result<T>>>
    where
        T: for<'de> Deserialize<'de>,
        P: Serialize,
    {
        Ok(self
            .send_batch(method, batch_args)?
            .into_iter()
            .map(|resp| {
                let resp = resp.ok_or(Error::Internal("Missing response in JSON-RPC batch"))?;
                resp.result::<T>().map_err(Error::from)
            })
            .collect())
    }

    /// Mixed-method batch: each `(method, args)` pair becomes one request
    /// in a single round-trip. Each result is independently parsed by the
    /// caller using its own `T`. Outer `Result` fails on transport errors;
    /// inner `Result`s fail on per-item RPC errors.
    pub fn call_mixed_batch(&self, calls: &[RpcCall]) -> Result<Vec<Result<Box<RawValue>>>> {
        let client = &self.client.read().0;
        let requests: Vec<Request> = calls
            .iter()
            .map(|call| client.build_request(call.method, Some(&call.params)))
            .collect();

        let responses = client
            .send_batch(&requests)
            .map_err(|e| match is_auth_rejection(&e) {
                true => Error::RpcAuthFailed,
                false => Error::Parse(format!("mixed batch failed: {e}")),
            })?;

        Ok(responses
            .into_iter()
            .map(|resp| {
                let resp = resp.ok_or(Error::Internal("Missing response in JSON-RPC batch"))?;
                resp.result::<Box<RawValue>>().map_err(Error::from)
            })
            .collect())
    }
}

/// Lost connections and the node's warm-up or overload answers.
fn is_retriable(error: &JsonRpcError) -> bool {
    match error {
        JsonRpcError::Rpc(e) => e.code == -32600 || e.code == 401 || e.code == -28,
        JsonRpcError::Transport(_) => true,
        _ => false,
    }
}

/// The node answered 401: it rejected the credentials.
fn is_auth_rejection(error: &JsonRpcError) -> bool {
    matches!(error, JsonRpcError::Transport(e)
        if matches!(e.downcast_ref(), Some(simple_http::Error::HttpErrorCode(401))))
}
