use std::time::Duration;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, PercentEncode, utf8_percent_encode};
use serde_json::Value;
use ureq::{Agent, Body, http::Response};

use crate::{
    manifest::{Operation, ParameterLocation},
    upstream_response::UpstreamResponse,
};

const MAX_UPSTREAM_URL_BYTES: usize = 32 * 1024;
const MAX_RESPONSE_BYTES: u64 = 8 * 1024 * 1024;
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(15);

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
            .build();
        Self {
            agent: Agent::new_with_config(config),
            api_url,
        }
    }

    /// The encoded upstream URL, checked against the URL limit.
    pub fn prepare(&self, operation: &Operation, arguments: &Value) -> Result<String, String> {
        let mut path = operation.http.path.clone();
        let mut query = String::new();

        for parameter in &operation.http.parameters {
            let Some(value) = arguments.get(parameter.name.as_str()) else {
                continue;
            };
            if value.is_null() {
                continue;
            }
            let values = parameter_values(value)?;
            match parameter.location {
                ParameterLocation::Path => {
                    // Client input: schema validation does not cover every shape, so check here.
                    let [value] = values.as_slice() else {
                        return Err(format!(
                            "path parameter {} must contain one scalar value",
                            parameter.name
                        ));
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

        if path.contains(['{', '}']) {
            return Err("a required path parameter is missing".to_string());
        }
        let url = format!("{}{path}{query}", self.api_url);
        if url.len() > MAX_UPSTREAM_URL_BYTES {
            return Err(format!(
                "upstream URL exceeds the {MAX_UPSTREAM_URL_BYTES}-byte limit"
            ));
        }

        Ok(url)
    }

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
        self.read_response(url, response)
    }

    fn read_response(
        &self,
        url: String,
        mut response: Response<Body>,
    ) -> Result<UpstreamResponse, String> {
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let cache_status = response
            .headers()
            .get("cf-cache-status")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let cache_age = response
            .headers()
            .get("age")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_RESPONSE_BYTES)
            .read_to_vec()
            .map_err(|error| {
                format!(
                    "Bitview API response read failed (limit {MAX_RESPONSE_BYTES} bytes): {error}"
                )
            })?;

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

fn parameter_values(value: &Value) -> Result<Vec<String>, String> {
    match value {
        Value::String(value) => Ok(vec![value.clone()]),
        Value::Number(value) => Ok(vec![value.to_string()]),
        Value::Bool(value) => Ok(vec![value.to_string()]),
        Value::Array(values) => values
            .iter()
            .map(|value| match value {
                Value::String(value) => Ok(value.clone()),
                Value::Number(value) => Ok(value.to_string()),
                Value::Bool(value) => Ok(value.to_string()),
                _ => Err("URL parameter arrays may contain only scalar values".to_string()),
            })
            .collect(),
        Value::Null => Ok(Vec::new()),
        Value::Object(_) => Err("URL parameters may not be objects".to_string()),
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
