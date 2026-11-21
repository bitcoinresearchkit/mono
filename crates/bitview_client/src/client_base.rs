use std::{
    collections::HashMap,
    fmt::Display,
    io::ErrorKind,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use bitview_types::ErrorCode;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::de::{DeserializeOwned, IntoDeserializer, value};
use serde_json::Value;
use ureq::{
    Agent, Body, BodyWithConfig,
    http::{HeaderMap, Response, StatusCode},
};

use crate::{BitviewError, Result};

type Attempt<T> = std::result::Result<Result<T>, ureq::Error>;

/// Defaults to `http://localhost:3110` with a 30 s timeout.
#[derive(Debug, Clone)]
pub struct BitviewClientOptions {
    pub base_url: String,
    pub timeout_secs: u64,
}

impl Default for BitviewClientOptions {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3110".to_string(),
            timeout_secs: 30,
        }
    }
}

/// How long a server that is gone is passed over: a misconfigured `bitview-origin` costs at most
/// one retried request per period, and a restarted server is taken back after it.
const REFUSE: Duration = Duration::from_secs(600);

/// Reuses connections through `ureq::Agent`. When several servers answer one URL, each names
/// itself in `bitview-origin` and the requests after go to it, so what one server minted (a
/// diff's `since`) stays with it. Whatever such an answer says, 5xx included, is the server's own
/// and goes to the caller; a server that fails to answer (network, timeout) or answers without the
/// header is gone: it is passed over and a GET goes through the configured URL again. POSTs
/// dispatch once. Clones share the server.
#[derive(Debug, Clone)]
pub struct BitviewClientBase {
    agent: Agent,
    base_url: String,
    pin: Arc<Mutex<Pin>>,
}

/// The server requests go to, if not the configured URL, and the servers passed over until when.
#[derive(Debug, Default)]
struct Pin {
    server: Option<String>,
    refused: HashMap<String, Instant>,
}

impl BitviewClientBase {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_options(BitviewClientOptions {
            base_url: base_url.into(),
            ..Default::default()
        })
    }

    pub fn with_options(options: BitviewClientOptions) -> Self {
        let agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(options.timeout_secs)))
            .http_status_as_error(false)
            .build()
            .into();
        Self {
            agent,
            base_url: options.base_url.trim_end_matches('/').to_string(),
            pin: Arc::default(),
        }
    }

    fn pin(&self) -> std::sync::MutexGuard<'_, Pin> {
        self.pin.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn get<T>(
        &self,
        path: &str,
        read: impl Fn(BodyWithConfig<'_>) -> std::result::Result<T, ureq::Error>,
    ) -> Result<T> {
        let send = |base: &str| self.agent.get(format!("{base}{path}")).call();
        let pinned = self.pin().server.clone();
        if let Some(server) = pinned
            && let (true, Ok(result)) = self.answer(Some(&server), send(&server), &read)
        {
            return result;
        }
        send(&self.base_url)
            .and_then(|response| self.read(response, read))
            .map_err(|error| BitviewError::new(error.to_string()))?
    }

    /// Reads the `response` to a request. A pinned `server` keeps its pin only when the answer
    /// names it and reads as an answer (an HTTP error included), which `true` says; any other
    /// outcome refuses it. The one place the rule lives: a GET goes on through the configured
    /// URL after a refusal, a POST returns what it got.
    fn answer<T>(
        &self,
        pinned: Option<&str>,
        response: std::result::Result<Response<Body>, ureq::Error>,
        read: impl FnOnce(BodyWithConfig<'_>) -> std::result::Result<T, ureq::Error>,
    ) -> (bool, Attempt<T>) {
        let named = response.as_ref().is_ok_and(names_server);
        let attempt = response.and_then(|response| self.read(response, read));
        let kept = named && attempt.is_ok();
        if let Some(server) = pinned
            && !kept
        {
            self.refuse(server);
        }
        (kept, attempt)
    }

    fn refuse(&self, server: &str) {
        let mut pin = self.pin();
        pin.refused
            .insert(server.to_owned(), Instant::now() + REFUSE);
        if pin.server.as_deref() == Some(server) {
            pin.server = None;
        }
    }

    /// Pins the server the answer names, unless it's the configured URL or refused.
    fn follow(&self, headers: &HeaderMap) {
        let origin = headers
            .get("bitview-origin")
            .and_then(|value| value.to_str().ok())
            .map(|origin| origin.trim_end_matches('/'))
            .filter(|origin| origin.starts_with("https://") || origin.starts_with("http://"));
        if let Some(origin) = origin {
            let mut pin = self.pin();
            if origin == self.base_url {
                pin.server = None;
            } else if pin.refused.get(origin).is_none_or(|until| *until <= Instant::now()) {
                pin.server = Some(origin.to_owned());
            }
        }
    }

    fn http_error(status: StatusCode, text: String) -> BitviewError {
        // Fields read one by one: a code this client doesn't know keeps the server's message.
        let detail = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|mut body| body.get_mut("error").map(Value::take));
        let field = |name: &str| detail.as_ref().and_then(|detail| detail.get(name)).cloned();
        let code = field("code").and_then(|code| serde_json::from_value::<ErrorCode>(code).ok());
        let message = match field("message") {
            Some(Value::String(message)) => message,
            _ if text.is_empty() => status.to_string(),
            _ => text,
        };
        BitviewError {
            status: Some(status.as_u16()),
            code,
            message,
        }
    }

    pub fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        parse_json(self.get_bytes(path)?)
    }

    pub fn get_text(&self, path: &str) -> Result<String> {
        self.get(path, |body| body.read_to_string())
    }

    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        self.get(path, |body| body.read_to_vec())
    }

    /// POSTs `body` as `content_type`, reading the JSON answer.
    pub fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &str,
        content_type: &str,
    ) -> Result<T> {
        parse_json(self.post_bytes(path, body, content_type)?)
    }

    /// POSTs `body` as `content_type`, reading the text answer.
    pub fn post_text(&self, path: &str, body: &str, content_type: &str) -> Result<String> {
        self.post(path, body, content_type, |body| body.read_to_string())
    }

    /// POSTs `body` as `content_type`, reading the binary answer.
    pub fn post_bytes(&self, path: &str, body: &str, content_type: &str) -> Result<Vec<u8>> {
        self.post(path, body, content_type, |body| body.read_to_vec())
    }

    fn post<T>(
        &self,
        path: &str,
        body: &str,
        content_type: &str,
        read: impl FnOnce(BodyWithConfig<'_>) -> std::result::Result<T, ureq::Error>,
    ) -> Result<T> {
        let pinned = self.pin().server.clone();
        let base = pinned.as_deref().unwrap_or(&self.base_url);
        let response = self
            .agent
            .post(format!("{base}{path}"))
            .config()
            // A redirect would dispatch another request for this submission.
            .max_redirects(0)
            .build()
            .header("Content-Type", content_type)
            .send(body);
        let (_, attempt) = self.answer(pinned.as_deref(), response, read);
        attempt
            .map_err(|error| BitviewError::new(error.to_string()))
            .flatten()
    }

    /// The outer error is a failed transport. A received HTTP error or bad payload is the
    /// server's answer. The server bounds its responses (ureq's cap is below the full UTXO set).
    fn read<T>(
        &self,
        mut response: Response<Body>,
        read: impl FnOnce(BodyWithConfig<'_>) -> std::result::Result<T, ureq::Error>,
    ) -> Attempt<T> {
        let status = response.status();
        if !status.is_success() {
            let text = response.body_mut().read_to_string();
            if text.is_ok() {
                self.follow(response.headers());
            }
            return Ok(Err(Self::http_error(status, text.unwrap_or_default())));
        }

        let value = match read(response.body_mut().with_config().limit(u64::MAX)) {
            Ok(value) => value,
            Err(error) => {
                let payload_error =
                    matches!(&error, ureq::Error::Io(error) if error.kind() == ErrorKind::InvalidData);
                return if payload_error {
                    Ok(Err(BitviewError::new(error.to_string())))
                } else {
                    Err(error)
                };
            }
        };
        self.follow(response.headers());
        Ok(Ok(value))
    }
}

/// An answer carrying `bitview-origin` came from the server itself, whatever its status.
fn names_server(response: &Response<Body>) -> bool {
    response.headers().contains_key("bitview-origin")
}

fn parse_json<T: DeserializeOwned>(bytes: Vec<u8>) -> Result<T> {
    serde_json::from_slice(&bytes)
        .map_err(|error| BitviewError::new(format!("Unexpected JSON response: {error}")))
}

/// Decodes a typed text body (a hash, txid or hex string) with its type's string deserializer.
pub(crate) fn parse_text<T: DeserializeOwned>(text: String) -> Result<T> {
    T::deserialize(text.into_deserializer()).map_err(|error: value::Error| {
        BitviewError::new(format!("Unexpected text response: {error}"))
    })
}

/// Every byte but RFC 3986's unreserved characters is percent-encoded.
const ESCAPED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// `name=value`, the value percent-encoded.
pub(crate) fn query_pair(name: &str, value: impl Display) -> String {
    format!(
        "{name}={}",
        utf8_percent_encode(&value.to_string(), ESCAPED)
    )
}

/// A path segment, percent-encoded so `/`, `?` or `#` in a value stay inside it.
pub(crate) fn path_segment(value: impl Display) -> String {
    utf8_percent_encode(&value.to_string(), ESCAPED).to_string()
}
