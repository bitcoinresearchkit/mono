use std::time::Duration;

use serde_json::{Map, Value};
use ureq::{Agent, Body, http::Response};

use crate::{
    manifest::{Operation, ParameterLocation},
    prepared_request::PreparedRequest,
    upstream_response::UpstreamResponse,
};

const MAX_UPSTREAM_URL_BYTES: usize = 32 * 1024;
const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct Upstream {
    agent: Agent,
    api_bases: Vec<String>,
}

impl Upstream {
    pub fn new(api_bases: Vec<String>) -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(UPSTREAM_TIMEOUT))
            .http_status_as_error(false)
            .max_redirects(0)
            .user_agent(format!("brk-mcp/{}", env!("CARGO_PKG_VERSION")))
            .build();
        Self {
            agent: Agent::new_with_config(config),
            api_bases,
        }
    }

    pub fn prepare(
        &self,
        operation: &Operation,
        arguments: &Map<String, Value>,
    ) -> Result<PreparedRequest, String> {
        let mut path = operation.http.path.clone();
        let mut query = String::new();

        for parameter in &operation.http.parameters {
            let Some(value) = arguments.get(&parameter.name) else {
                continue;
            };
            if value.is_null() {
                continue;
            }
            let values = parameter_values(value)?;
            match parameter.location {
                ParameterLocation::Path => {
                    if values.len() != 1 {
                        return Err(format!(
                            "path parameter {} must contain one scalar value",
                            parameter.name
                        ));
                    }
                    path = path.replace(
                        &format!("{{{}}}", parameter.name),
                        &encode_component(&values[0]),
                    );
                }
                ParameterLocation::Query => {
                    for value in values {
                        query.push(if query.is_empty() { '?' } else { '&' });
                        query.push_str(&encode_component(&parameter.name));
                        query.push('=');
                        query.push_str(&encode_component(&value));
                    }
                }
            }
        }

        if path.contains('{') || path.contains('}') {
            return Err("a required path parameter is missing".to_string());
        }
        path.push_str(&query);
        let longest_base = self.api_bases.iter().map(String::len).max().unwrap_or(0);
        if longest_base.saturating_add(path.len()) > MAX_UPSTREAM_URL_BYTES {
            return Err(format!(
                "upstream URL exceeds the {MAX_UPSTREAM_URL_BYTES}-byte limit"
            ));
        }

        Ok(PreparedRequest { path })
    }

    pub fn fetch(&self, request: PreparedRequest) -> Result<UpstreamResponse, String> {
        let PreparedRequest { path } = request;
        let mut last_error = None;
        for api_base in &self.api_bases {
            let url = format!("{api_base}{path}");
            let response = self
                .agent
                .get(&url)
                .header(
                    "Accept",
                    "application/json, text/plain, text/csv, application/octet-stream",
                )
                .call();
            match response {
                Ok(response) => return self.read_response(url, response),
                Err(error) => last_error = Some(error),
            }
        }
        Err(format!(
            "Bitview API request failed: {}",
            last_error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "no API base URL configured".to_string())
        ))
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
            .limit(u64::try_from(MAX_RESPONSE_BYTES).unwrap_or(u64::MAX))
            .read_to_vec()
            .map_err(|error| {
                format!(
                    "Bitview API response could not be read within the {}-byte limit: {error}",
                    MAX_RESPONSE_BYTES
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

fn encode_component(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[(byte >> 4) as usize]));
            encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
        }
    }
    encoded
}
