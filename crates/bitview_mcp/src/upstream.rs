use std::{io::Read, time::Duration};

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, PercentEncode, utf8_percent_encode};
use serde_json::{Map, Value};
use ureq::Agent;

use crate::{
    manifest::{Operation, ParameterLocation},
    upstream_response::UpstreamResponse,
};

const MAX_UPSTREAM_URL_BYTES: usize = 32 * 1024;
/// The largest answer a call returns: every default answer fits (the largest, a raw URPD of all coins, is 1.3 MB), and
/// more is past what an assistant reads.
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
/// The API gives a request 5 s, then answers 504 itself; the rest is for the connection and the body.
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(8);
/// Calls in flight at once. Each holds a blocking thread, an upstream connection (about 100 KB of buffers, kept open
/// between calls: 6 MiB for all), and at most its answer twice, raw and as JSON text: 256 MiB for all.
pub const MAX_CALLS: usize = 64;

pub struct Upstream {
    agent: Agent,
    api_url: String,
}

impl Upstream {
    pub fn new(api_url: String) -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(UPSTREAM_TIMEOUT))
            .http_status_as_error(false)
            .max_redirects(0)
            .user_agent(format!("brk-mcp/{}", env!("CARGO_PKG_VERSION")))
            // Every call in flight holds a connection, which holds two of each buffer (TCP and TLS):
            // one TLS record in, and the longest request out.
            .input_buffer_size(16 * 1024)
            .output_buffer_size(MAX_UPSTREAM_URL_BYTES + 1024)
            // Every call goes to the one API host. ureq keeps 3 idle connections per host by default: with more calls
            // in flight, the rest close after use and the next calls open (and TLS-handshake) new ones.
            .max_idle_connections(MAX_CALLS)
            .max_idle_connections_per_host(MAX_CALLS)
            .build();
        Self {
            agent: Agent::new_with_config(config),
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
        let tool = &operation.tool.name;
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
    pub fn fetch(&self, url: String) -> Result<UpstreamResponse, String> {
        let response = self
            .agent
            .get(&url)
            .header(
                "Accept",
                "application/json, text/plain, text/csv, application/octet-stream",
            )
            .call()
            .map_err(|error| format!("Bitview API request failed: {error}"))?;
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
        let mut body = Vec::new();
        response
            .into_body()
            .into_reader()
            .take(MAX_RESPONSE_BYTES as u64 + 1)
            .read_to_end(&mut body)
            .map_err(|error| format!("Bitview API response read failed: {error}"))?;
        if body.len() > MAX_RESPONSE_BYTES {
            return Err(format!(
                "This answer is over {} MiB, the most a call returns: ask for less, with a shorter range, a smaller \
                 limit or a coarser resolution",
                MAX_RESPONSE_BYTES / (1024 * 1024)
            ));
        }

        Ok(UpstreamResponse {
            url,
            status,
            content_type,
            body,
            cache_status,
            cache_age,
        })
    }
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
