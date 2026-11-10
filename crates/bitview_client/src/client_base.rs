use std::{fmt::Display, time::Duration};

use bitview_types::ErrorCode;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::de::{DeserializeOwned, IntoDeserializer, value};
use serde_json::Value;
use ureq::{Agent, Body, BodyWithConfig, http::Response};

use crate::{BitviewError, Result};

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

/// Reuses connections through `ureq::Agent`.
#[derive(Debug, Clone)]
pub struct BitviewClientBase {
    agent: Agent,
    base_url: String,
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

    pub fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        Self::read(self.agent.get(&self.url(path)).call(), |_, body| {
            body.read_json()
        })
    }

    /// JSON 200 or a declared empty 204; an empty 200 is still invalid JSON.
    pub fn get_optional_json<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        Self::read_optional_json(self.agent.get(&self.url(path)).call())
    }

    pub fn get_text(&self, path: &str) -> Result<String> {
        Self::read(self.agent.get(&self.url(path)).call(), |_, body| {
            body.read_to_string()
        })
    }

    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        Self::read(self.agent.get(&self.url(path)).call(), |_, body| {
            body.read_to_vec()
        })
    }

    /// POSTs `body` as `content_type`, reading the JSON answer.
    pub fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &str,
        content_type: &str,
    ) -> Result<T> {
        Self::read(self.post(path, body, content_type), |_, body| body.read_json())
    }

    /// JSON 200 or a declared empty 204, with the same error handling as other requests.
    pub fn post_optional_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &str,
        content_type: &str,
    ) -> Result<Option<T>> {
        Self::read_optional_json(self.post(path, body, content_type))
    }

    /// POSTs `body` as `content_type`, reading the text answer.
    pub fn post_text(&self, path: &str, body: &str, content_type: &str) -> Result<String> {
        Self::read(self.post(path, body, content_type), |_, body| {
            body.read_to_string()
        })
    }

    /// POSTs `body` as `content_type`, reading the binary answer.
    pub fn post_bytes(&self, path: &str, body: &str, content_type: &str) -> Result<Vec<u8>> {
        Self::read(self.post(path, body, content_type), |_, body| body.read_to_vec())
    }

    fn post(
        &self,
        path: &str,
        body: &str,
        content_type: &str,
    ) -> std::result::Result<Response<Body>, ureq::Error> {
        self.agent
            .post(&self.url(path))
            .header("Content-Type", content_type)
            .send(body)
    }

    fn read_optional_json<T: DeserializeOwned>(
        response: std::result::Result<Response<Body>, ureq::Error>,
    ) -> Result<Option<T>> {
        Self::read(response, |status, body| {
            if status == 204 { Ok(None) } else { body.read_json() }
        })
    }

    /// Reads a successful response's whole body: the server bounds its responses, so the
    /// client adds no cap (ureq's default stops at 10 MiB, below the full UTXO set).
    fn read<T>(
        response: std::result::Result<Response<Body>, ureq::Error>,
        read: impl FnOnce(u16, BodyWithConfig<'_>) -> std::result::Result<T, ureq::Error>,
    ) -> Result<T> {
        let mut response = Self::check(response)?;
        read(response.status().as_u16(), response.body_mut().with_config().limit(u64::MAX))
            .map_err(|e| BitviewError::new(e.to_string()))
    }
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
