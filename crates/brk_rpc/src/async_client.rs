use std::{path::PathBuf, time::Duration};

use bitcoin::Txid as BitcoinTxid;
use brk_error::{Error, Result};
use brk_types::{Height, Txid};
use http_body_util::{BodyExt, Full, Limited};
use hyper::{
    Request, StatusCode,
    body::Bytes,
    client::conn::http1,
    header::{AUTHORIZATION, HOST, HeaderValue},
};
use hyper_util::rt::TokioIo;
use parking_lot::RwLock;
use serde::de::DeserializeOwned;
use serde_json::{json, to_vec};
use tokio::{fs as TokioFs, net::TcpStream, pin, select, time::timeout};

use crate::{
    Auth,
    auth::{basic, cookie_io_error, read_cookie},
    endpoint::Endpoint,
    rpc_client,
    rpc_response::{self, MAX_RESPONSE_BYTES},
};

/// Async RPC transport for request handlers. Each call opens its own connection: none is kept, so
/// a submission, which can't be replayed, never meets one the node closed while it sat idle (Core
/// drops it after `-rpcservertimeout`, 30 s by default). Callers apply their request deadline;
/// dropping a call closes its socket.
pub struct AsyncClient {
    endpoint: Endpoint,
    cookie: Option<PathBuf>,
    authorization: RwLock<Option<HeaderValue>>,
}

impl AsyncClient {
    pub(crate) fn new(url: &str, auth: Auth) -> Result<Self> {
        let endpoint = Endpoint::parse(url)?;
        let (cookie, header) = match auth {
            Auth::None => (None, None),
            Auth::UserPass(user, password) => {
                (None, Some(authorization(&format!("{user}:{password}"))?))
            }
            Auth::CookieFile(path) => {
                let header = authorization(&read_cookie(&path).map_err(Error::IO)?)?;
                (Some(path), Some(header))
            }
        };
        Ok(Self {
            endpoint,
            cookie,
            authorization: RwLock::new(header),
        })
    }

    pub async fn get_last_height(&self) -> Result<Height> {
        let height: u64 = self
            .call(Bytes::from_static(
                br#"{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}"#,
            ))
            .await?;
        let height = u32::try_from(height)
            .map_err(|_| Error::Internal("node height exceeds supported range"))?;
        Ok(Height::new(height))
    }

    /// Submits once. A transport failure after the request left, or dropping the call then,
    /// leaves the outcome unknown: the node may hold the transaction.
    pub async fn send_raw_transaction(&self, hex: &str) -> Result<Txid> {
        let body = to_vec(&json!({
            "jsonrpc": "2.0", "id": 1, "method": "sendrawtransaction", "params": [hex],
        }))?;
        self.call::<BitcoinTxid>(body.into())
            .await
            .map(Txid::from)
            .map_err(rpc_client::transaction_error)
    }

    /// One answered exchange. Credentials the node rejects outright are read again and the call
    /// replayed once (a restarted node wrote a new cookie): that rejection ran nothing.
    async fn call<T: DeserializeOwned>(&self, body: Bytes) -> Result<T> {
        let mut refresh = true;
        loop {
            let mut request = Request::post(self.endpoint.path.as_str())
                .header(HOST, self.endpoint.authority.as_str())
                .header("content-type", "application/json");
            let header = self.authorization.read().clone();
            if let Some(header) = &header {
                request = request.header(AUTHORIZATION, header);
            }
            let request = request
                .body(Full::new(body.clone()))
                .map_err(|_| Error::Internal("invalid node RPC request"))?;
            let (status, bytes) = self.exchange(request).await?;
            if status == StatusCode::UNAUTHORIZED {
                if refresh && let Some(path) = &self.cookie {
                    let cookie = TokioFs::read_to_string(path)
                        .await
                        .map_err(|error| Error::IO(cookie_io_error(path, error)))?;
                    let updated = authorization(cookie.trim())?;
                    if header.as_ref() != Some(&updated) {
                        *self.authorization.write() = Some(updated);
                        refresh = false;
                        continue;
                    }
                }
                return Err(Error::RpcAuthFailed);
            }
            if status == StatusCode::FORBIDDEN {
                return Err(Error::RpcForbidden);
            }
            return rpc_response::decode(status.is_success(), &bytes);
        }
    }

    /// One request on a connection of its own. The connection is driven here, by no background
    /// task, so it closes with this future.
    async fn exchange(&self, request: Request<Full<Bytes>>) -> Result<(StatusCode, Bytes)> {
        let socket = timeout(
            Duration::from_secs(3),
            TcpStream::connect((self.endpoint.host.as_str(), self.endpoint.port)),
        )
        .await
        .map_err(|_| Error::Internal("node RPC connection timed out"))??;
        let (mut sender, mut connection) = http1::handshake(TokioIo::new(socket))
            .await
            .map_err(|_| Error::Internal("node RPC handshake failed"))?;
        let answer = async {
            let response = sender.send_request(request).await.map_err(|error| {
                Error::Internal(if error.is_parse() || error.is_user() {
                    "invalid node RPC HTTP exchange"
                } else {
                    "node RPC transport failed"
                })
            })?;
            let status = response.status();
            let bytes = Limited::new(response.into_body(), MAX_RESPONSE_BYTES)
                .collect()
                .await
                .map_err(|_| Error::Internal("node RPC response unreadable or too large"))?
                .to_bytes();
            Ok((status, bytes))
        };
        pin!(answer);
        select! {
            biased;
            answer = &mut answer => answer,
            _ = &mut connection => {
                // Dropping the ended connection releases a request it left pending.
                drop(connection);
                answer.await
            }
        }
    }
}

fn authorization(credentials: &str) -> Result<HeaderValue> {
    let mut header = HeaderValue::from_str(&basic(credentials))
        .map_err(|_| Error::Internal("invalid node RPC credentials"))?;
    header.set_sensitive(true);
    Ok(header)
}
