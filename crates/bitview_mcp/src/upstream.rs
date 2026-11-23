use std::{
    error::Error,
    fmt::Write as _,
    io::{self, Write},
    time::Duration,
};

use axum::http::{Request, header::CONTENT_ENCODING};
use flate2::write::MultiGzDecoder;
use http_body_util::{BodyExt, Empty};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, PercentEncode, utf8_percent_encode};
use serde_json::{Map, Value};
use tokio::time::timeout;

use crate::{
    manifest::{Operation, ParameterLocation},
    upstream_response::UpstreamResponse,
    upstream_transport::{Client, client},
};

const MAX_UPSTREAM_URL_BYTES: usize = 32 * 1024;
/// The largest answer a call returns: every default answer fits (the largest, a raw URPD of all coins, is 1.3 MB), and
/// more is past what an assistant reads.
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
/// The API gives a request 5 s, then answers 504 itself; the rest is for the connection and the body.
pub const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(8);
/// Calls in flight at once. Each holds an upstream connection and up to 2 MiB of response data.
pub const MAX_CALLS: usize = 64;

pub struct Upstream {
    client: Client,
    api_url: String,
}

impl Upstream {
    pub fn new(api_url: String) -> Self {
        Self {
            client: client(),
            api_url,
        }
    }

    /// The upstream URL for a tool call. The API judges the values; an error here is one only the bridge can see, and
    /// names the argument and the fix for the model to act on.
    pub fn prepare(
        &self,
        operation: &Operation,
        arguments: &Map<String, Value>,
    ) -> Result<String, String> {
        let tool = &operation.name;
        let parameters = &operation.http.parameters;
        // Dropped silently, a misspelled filter would answer with unfiltered data.
        if let Some(unknown) = arguments.keys().find(|name| {
            !parameters
                .iter()
                .any(|parameter| parameter.argument == **name)
        }) {
            let known = parameters
                .iter()
                .map(|parameter| parameter.argument.as_str())
                .collect::<Vec<_>>();
            return Err(if known.is_empty() {
                format!("{tool} takes no arguments: remove `{unknown}`")
            } else {
                format!(
                    "{tool} has no argument `{unknown}`: its arguments are {}",
                    known.join(", ")
                )
            });
        }

        let mut path = operation.http.path.clone();
        let mut query = String::new();
        for parameter in parameters {
            let argument = &parameter.argument;
            let values = match arguments.get(argument) {
                None | Some(Value::Null) => Some(Vec::new()),
                Some(Value::Array(values)) => values.iter().map(scalar).collect(),
                Some(value) => scalar(value).map(|value| vec![value]),
            }
            .ok_or_else(|| {
                format!(
                    "{tool}'s `{argument}` takes a string, a number or a boolean, or a list of them"
                )
            })?;
            match parameter.location {
                ParameterLocation::Path => {
                    let [value] = values.as_slice() else {
                        return Err(format!("{tool} needs one value for `{argument}`"));
                    };
                    path = path.replace(
                        &format!("{{{}}}", parameter.name),
                        &encode_component(value).to_string(),
                    );
                }
                ParameterLocation::Query => {
                    for value in values {
                        query.push(if query.is_empty() { '?' } else { '&' });
                        query.extend(encode_component(&parameter.name));
                        query.push('=');
                        query.extend(encode_component(&value));
                    }
                }
            }
        }

        let url = format!("{}{path}{query}", self.api_url);
        if url.len() > MAX_UPSTREAM_URL_BYTES {
            return Err(format!(
                "{tool}'s arguments make a URL over {MAX_UPSTREAM_URL_BYTES} bytes: pass fewer or shorter values"
            ));
        }
        Ok(url)
    }

    /// The whole response. A body over `MAX_RESPONSE_BYTES` is an error naming the fix.
    pub async fn fetch(&self, url: String) -> Result<UpstreamResponse, String> {
        timeout(UPSTREAM_TIMEOUT, self.read(url))
            .await
            .map_err(|_| "Bitview API request timed out".to_string())?
    }

    async fn read(&self, url: String) -> Result<UpstreamResponse, String> {
        let request = Request::get(&url)
            .header("User-Agent", concat!("brk-mcp/", env!("CARGO_PKG_VERSION")))
            .header("Accept-Encoding", "gzip")
            .header(
                "Accept",
                "application/json, text/plain, text/csv, application/octet-stream",
            )
            .body(Empty::new())
            .map_err(|error| failure("Bitview API request failed", &error))?;
        let response = self
            .client
            .request(request)
            .await
            .map_err(|error| failure("Bitview API request failed", &error))?;
        let header = |name| {
            response
                .headers()
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
        };
        let status = response.status().as_u16();
        let content_type =
            header("content-type").unwrap_or_else(|| "application/octet-stream".to_string());
        let cache_status = header("cf-cache-status");
        let cache_age = header("age");
        let gzip = response
            .headers()
            .get(CONTENT_ENCODING)
            .is_some_and(|encoding| encoding == "gzip");
        let mut decoder = gzip.then(|| MultiGzDecoder::new(CappedBody(Vec::new())));
        let mut body = CappedBody(Vec::new());
        let mut incoming = response.into_body();
        while let Some(frame) = incoming.frame().await {
            let frame =
                frame.map_err(|error| failure("Bitview API response read failed", &error))?;
            if let Ok(data) = frame.into_data() {
                match &mut decoder {
                    Some(decoder) => decoder.write_all(&data),
                    None => body.write_all(&data),
                }
                .map_err(body_error)?;
            }
        }
        if let Some(decoder) = decoder {
            body = decoder.finish().map_err(body_error)?;
        }

        Ok(UpstreamResponse {
            url,
            status,
            content_type,
            body: body.0,
            cache_status,
            cache_age,
        })
    }
}

struct CappedBody(Vec<u8>);

impl Write for CappedBody {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if data.len() > MAX_RESPONSE_BYTES - self.0.len() {
            return Err(io::ErrorKind::FileTooLarge.into());
        }
        self.0.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn body_error(error: io::Error) -> String {
    if error.kind() == io::ErrorKind::FileTooLarge {
        format!(
            "This answer is over {} MiB, the most a call returns: ask for less, with a shorter range, a smaller \
             limit or a coarser resolution",
            MAX_RESPONSE_BYTES / (1024 * 1024)
        )
    } else {
        failure("Bitview API response read failed", &error)
    }
}

/// `what: error`, then each cause down its `source()` chain. hyper-util's own text is only "client error (Connect)",
/// the refused connection or bad certificate under it is what the caller needs. A cause whose text repeats the one
/// above it adds nothing and is skipped.
fn failure(what: &str, error: &impl Error) -> String {
    let mut message = format!("{what}: {error}");
    let mut last = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        let text = cause.to_string();
        if text != last {
            let _ = write!(message, ": {text}");
            last = text;
        }
        source = cause.source();
    }
    message
}

/// A scalar as a URL writes it; none for an object or a nested list.
fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Null | Value::Array(_) | Value::Object(_) => None,
    }
}

/// Every byte but RFC 3986's unreserved characters is percent-encoded.
const ESCAPED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn encode_component(value: &str) -> PercentEncode<'_> {
    utf8_percent_encode(value, ESCAPED)
}
