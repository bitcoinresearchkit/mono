use std::{thread::sleep, time::Duration};

use brk_error::Error;
use corepc_jsonrpc::{Client as JsonRpcClient, Request, error::Error as JsonRpcError, simple_http};
use parking_lot::RwLock;
use serde::Deserialize;
use serde_json::{Value, value::RawValue};
use tracing::info;

use crate::Auth;

#[derive(Debug)]
pub(crate) struct ClientInner {
    url: String,
    auth: Auth,
    client: RwLock<JsonRpcClient>,
    max_retries: usize,
    retry_delay: Duration,
}

impl ClientInner {
    pub(crate) fn new(
        url: &str,
        auth: Auth,
        max_retries: usize,
        retry_delay: Duration,
    ) -> brk_error::Result<Self> {
        let client = Self::create_client(url, &auth)?;
        Ok(Self {
            url: url.to_string(),
            auth,
            client: RwLock::new(client),
            max_retries,
            retry_delay,
        })
    }

    /// Builds a `jsonrpc::Client` using the `simple_http` transport, which
    /// keeps a single pooled TCP socket with reconnect-on-failure. The
    /// upstream `corepc-client` hard-wires `bitreq_http` (one TCP connect
    /// per request), which collapses under concurrent load.
    fn create_client(url: &str, auth: &Auth) -> brk_error::Result<JsonRpcClient> {
        let builder = simple_http::Builder::new()
            .url(url)
            .map_err(|e| Error::Parse(format!("bad rpc url: {e}")))?
            .timeout(Duration::from_secs(60));
        let builder = match auth {
            Auth::None => builder,
            Auth::UserPass(u, p) => builder.auth(u.clone(), Some(p.clone())),
            Auth::CookieFile(path) => {
                let cookie = std::fs::read_to_string(path)?;
                builder.cookie_auth(cookie.trim())
            }
        };
        Ok(JsonRpcClient::with_transport(builder.build()))
    }

    fn recreate(&self) -> brk_error::Result<()> {
        *self.client.write() = Self::create_client(&self.url, &self.auth)?;
        Ok(())
    }

    fn is_retriable(error: &JsonRpcError) -> bool {
        match error {
            JsonRpcError::Rpc(e) => e.code == -32600 || e.code == 401 || e.code == -28,
            JsonRpcError::Transport(_) => true,
            _ => false,
        }
    }

    pub(crate) fn call_with_retry<T>(&self, method: &str, args: &[Value]) -> brk_error::Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let raw = serde_json::value::to_raw_value(args).map_err(Error::from)?;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                info!(
                    "Trying to reconnect to Bitcoin Core (attempt {}/{})",
                    attempt, self.max_retries
                );
                self.recreate().ok();
                sleep(self.retry_delay);
            }

            match self.client.read().call::<T>(method, Some(&raw)) {
                Ok(value) => {
                    if attempt > 0 {
                        info!(
                            "Successfully reconnected to Bitcoin Core after {} attempts",
                            attempt
                        );
                    }
                    return Ok(value);
                }
                Err(e) if Self::is_retriable(&e) => {
                    if attempt == 0 {
                        info!("Lost connection to Bitcoin Core, reconnecting...");
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        info!(
            "Could not reconnect to Bitcoin Core after {} attempts",
            self.max_retries + 1
        );
        Err(JsonRpcError::Rpc(corepc_jsonrpc::error::RpcError {
            code: -1,
            message: "Max retries exceeded".to_string(),
            data: None,
        })
        .into())
    }

    pub(crate) fn call_once<T>(&self, method: &str, args: &[Value]) -> brk_error::Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let raw = serde_json::value::to_raw_value(args).map_err(Error::from)?;
        Ok(self.client.read().call::<T>(method, Some(&raw))?)
    }

    /// Send a batch of calls sharing `method`, one set of args per request.
    /// No retry: the caller decides batch sizing and failure semantics.
    pub(crate) fn call_batch<T>(
        &self,
        method: &str,
        batch_args: impl IntoIterator<Item = Vec<Value>>,
    ) -> brk_error::Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let params: Vec<Box<RawValue>> = batch_args
            .into_iter()
            .map(|args| serde_json::value::to_raw_value(&args).map_err(Error::from))
            .collect::<brk_error::Result<Vec<_>>>()?;

        let client = self.client.read();
        let requests: Vec<Request> = params
            .iter()
            .map(|p| client.build_request(method, Some(p)))
            .collect();

        let responses = client
            .send_batch(&requests)
            .map_err(|e| Error::Parse(format!("batch {method} failed: {e}")))?;

        responses
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
    pub(crate) fn call_batch_per_item<T>(
        &self,
        method: &str,
        batch_args: impl IntoIterator<Item = Vec<Value>>,
    ) -> brk_error::Result<Vec<brk_error::Result<T>>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let params: Vec<Box<RawValue>> = batch_args
            .into_iter()
            .map(|args| serde_json::value::to_raw_value(&args).map_err(Error::from))
            .collect::<brk_error::Result<Vec<_>>>()?;

        let client = self.client.read();
        let requests: Vec<Request> = params
            .iter()
            .map(|p| client.build_request(method, Some(p)))
            .collect();

        let responses = client
            .send_batch(&requests)
            .map_err(|e| Error::Parse(format!("batch {method} failed: {e}")))?;

        Ok(responses
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
    pub(crate) fn call_mixed_batch(
        &self,
        requests: &[(&str, Vec<Value>)],
    ) -> brk_error::Result<Vec<brk_error::Result<Box<RawValue>>>> {
        let params: Vec<Box<RawValue>> = requests
            .iter()
            .map(|(_, args)| serde_json::value::to_raw_value(args).map_err(Error::from))
            .collect::<brk_error::Result<Vec<_>>>()?;

        let client = self.client.read();
        let built: Vec<Request> = requests
            .iter()
            .zip(&params)
            .map(|((method, _), p)| client.build_request(method, Some(p)))
            .collect();

        let responses = client
            .send_batch(&built)
            .map_err(|e| Error::Parse(format!("mixed batch failed: {e}")))?;

        Ok(responses
            .into_iter()
            .map(|resp| {
                let resp = resp.ok_or(Error::Internal("Missing response in JSON-RPC batch"))?;
                resp.result::<Box<RawValue>>().map_err(Error::from)
            })
            .collect())
    }
}
