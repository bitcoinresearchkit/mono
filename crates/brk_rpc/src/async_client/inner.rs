use std::{fs, path::PathBuf};

use base64::{Engine, engine::general_purpose::STANDARD};
use bitcoin::Txid as BitcoinTxid;
use brk_error::{Error, Result};
use brk_types::{Height, Txid};
use http_body_util::Full;
use hyper::{
    Request, StatusCode, Uri,
    body::Bytes,
    header::{AUTHORIZATION, HOST, HeaderValue},
};
use parking_lot::RwLock;
use serde::de::DeserializeOwned;
use serde_json::{json, to_vec};
use tokio::{fs as TokioFs, sync::Mutex};

use super::connection::{Connection, Exchange};
use crate::{Auth, auth::cookie_error, rpc_client, rpc_response};

pub struct Inner {
    connection: Mutex<Option<Connection>>,
    uri: Uri,
    cookie: Option<PathBuf>,
    authorization: RwLock<Option<HeaderValue>>,
}

impl Inner {
    pub fn new(url: &str, auth: Auth) -> Result<Self> {
        let uri: Uri = url
            .parse()
            .map_err(|_| Error::Internal("invalid node RPC URL"))?;
        if uri.scheme_str() != Some("http") || uri.host().is_none() {
            return Err(Error::Internal("node RPC requires an HTTP URL"));
        }
        let (cookie, header) = match auth {
            Auth::None => (None, None),
            Auth::UserPass(user, password) => {
                (None, Some(authorization(&format!("{user}:{password}"))?))
            }
            Auth::CookieFile(path) => {
                let cookie =
                    fs::read_to_string(&path).map_err(|error| cookie_error(&path, error))?;
                let header = authorization(cookie.trim())?;
                (Some(path), Some(header))
            }
        };
        Ok(Self {
            connection: Mutex::new(None),
            uri,
            cookie,
            authorization: RwLock::new(header),
        })
    }

    pub async fn get_last_height(&self) -> Result<Height> {
        let height: u64 = self
            .call(
                Bytes::from_static(
                    br#"{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}"#,
                ),
                true,
            )
            .await?;
        let height = u32::try_from(height)
            .map_err(|_| Error::Internal("node height exceeds supported range"))?;
        Ok(Height::new(height))
    }

    pub async fn send_raw_transaction(&self, hex: &str) -> Result<Txid> {
        let body = to_vec(&json!({
            "jsonrpc": "2.0", "id": 1, "method": "sendrawtransaction", "params": [hex],
        }))?;
        self.call::<BitcoinTxid>(body.into(), false)
            .await
            .map(Txid::from)
            .map_err(rpc_client::transaction_error)
    }

    /// Only read-only calls may retry a disconnected exchange. An explicit
    /// authentication rejection can refresh credentials once for either call.
    async fn call<T: DeserializeOwned>(&self, body: Bytes, mut reconnect: bool) -> Result<T> {
        let mut refresh = true;
        loop {
            let mut request =
                Request::post(self.uri.path_and_query().map_or("/", |path| path.as_str()))
                    .header(
                        HOST,
                        self.uri
                            .authority()
                            .expect("validated RPC authority")
                            .as_str(),
                    )
                    .header("content-type", "application/json");
            let header = self.authorization.read().clone();
            if let Some(header) = &header {
                request = request.header(AUTHORIZATION, header);
            }
            let request = request
                .body(Full::new(body.clone()))
                .map_err(|_| Error::Internal("invalid node RPC request"))?;
            let (status, bytes) = {
                let mut idle = self.connection.lock().await;
                let reused = idle.is_some();
                // Taking ownership makes cancellation close the socket, with no background task.
                let mut connection = match idle.take() {
                    Some(connection) => connection,
                    None => Connection::connect(&self.uri).await?,
                };
                let (status, bytes) = match connection.request(request).await? {
                    Exchange::Complete(status, bytes) => (status, bytes),
                    Exchange::Disconnected if reused && reconnect => {
                        // Only this read-only height call may be replayed, at most once.
                        reconnect = false;
                        continue;
                    }
                    Exchange::Disconnected => {
                        return Err(Error::Internal("node RPC transport failed"));
                    }
                };
                if !connection.is_closed().await {
                    *idle = Some(connection);
                }
                (status, bytes)
            };
            if status == StatusCode::UNAUTHORIZED {
                if refresh && let Some(path) = &self.cookie {
                    let cookie = TokioFs::read_to_string(path)
                        .await
                        .map_err(|error| cookie_error(path, error))?;
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
}

fn authorization(credentials: &str) -> Result<HeaderValue> {
    let mut header = HeaderValue::from_str(&format!("Basic {}", STANDARD.encode(credentials)))
        .map_err(|_| Error::Internal("invalid node RPC credentials"))?;
    header.set_sensitive(true);
    Ok(header)
}
