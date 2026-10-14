use std::{
    fmt::{Display, Write},
    time::Duration,
};

use bitview_types::ErrorCode;
use serde::de::{DeserializeOwned, IntoDeserializer, value};
use serde_json::Value;
use ureq::{Agent, Body, BodyWithConfig, http::Response};

use crate::{BitviewError, Result};

/// Options for configuring the Bitview client.
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

/// Base HTTP client for making requests. Reuses connections via ureq::Agent.
#[derive(Debug, Clone)]
pub struct BitviewClientBase {
    agent: Agent,
    base_url: String,
}

impl BitviewClientBase {
    /// Create a new client with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_options(BitviewClientOptions {
            base_url: base_url.into(),
            ..Default::default()
        })
    }

    /// Create a new client with options.
    pub fn with_options(options: BitviewClientOptions) -> Self {
        let agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(options.timeout_secs)))
            .http_status_as_error(false)
            .build()
            .into();
        Self {
            agent,
            base_url: options.base_url.trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// Passes a successful answer through; an error status becomes a `BitviewError` with the
    /// server's code and message when it sent an error body.
    fn check(response: std::result::Result<Response<Body>, ureq::Error>) -> Result<Response<Body>> {
        let mut response = response.map_err(|e| BitviewError::new(e.to_string()))?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let text = response.body_mut().read_to_string().unwrap_or_default();
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
        Err(BitviewError {
            status: Some(status.as_u16()),
            code,
            message,
        })
    }

    /// Make a GET request and deserialize JSON response.
    pub fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        Self::read(self.agent.get(&self.url(path)).call(), |body| {
            body.read_json()
        })
    }

    /// Make a GET request and return raw text response.
    pub fn get_text(&self, path: &str) -> Result<String> {
        Self::read(self.agent.get(&self.url(path)).call(), |body| {
            body.read_to_string()
        })
    }

    /// Make a GET request and return raw bytes response.
    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        Self::read(self.agent.get(&self.url(path)).call(), |body| {
            body.read_to_vec()
        })
    }

    /// Make a POST request and deserialize JSON response.
    pub fn post_json<T: DeserializeOwned>(&self, path: &str, body: &str) -> Result<T> {
        Self::read(self.agent.post(&self.url(path)).send(body), |body| {
            body.read_json()
        })
    }

    /// Make a POST request and return raw text response.
    pub fn post_text(&self, path: &str, body: &str) -> Result<String> {
        Self::read(self.agent.post(&self.url(path)).send(body), |body| {
            body.read_to_string()
        })
    }

    /// Make a POST request and return raw bytes response.
    pub fn post_bytes(&self, path: &str, body: &str) -> Result<Vec<u8>> {
        Self::read(self.agent.post(&self.url(path)).send(body), |body| {
            body.read_to_vec()
        })
    }

    /// Reads a successful response's whole body: the server bounds its responses, so the
    /// client adds no cap (ureq's default stops at 10 MiB, below the full UTXO set).
    fn read<T>(
        response: std::result::Result<Response<Body>, ureq::Error>,
        read: impl FnOnce(BodyWithConfig<'_>) -> std::result::Result<T, ureq::Error>,
    ) -> Result<T> {
        let mut response = Self::check(response)?;
        read(response.body_mut().with_config().limit(u64::MAX))
            .map_err(|e| BitviewError::new(e.to_string()))
    }
}

/// Decodes a typed text body (a hash, txid or hex string) with its type's string deserializer.
pub(crate) fn parse_text<T: DeserializeOwned>(text: String) -> Result<T> {
    T::deserialize(text.into_deserializer()).map_err(|error: value::Error| {
        BitviewError::new(format!("Unexpected text response: {error}"))
    })
}

/// `name=value`, the value percent-encoded (RFC 3986 unreserved characters pass through).
pub(crate) fn query_pair(name: &str, value: impl Display) -> String {
    let value = value.to_string();
    let mut pair = format!("{name}=");
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            pair.push(char::from(byte));
        } else {
            write!(pair, "%{byte:02X}").unwrap();
        }
    }
    pair
}
