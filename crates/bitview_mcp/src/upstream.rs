use std::time::Duration;

use serde_json::{Map, Value};

use crate::manifest::{Operation, ParameterLocation};

const MAX_UPSTREAM_URL_BYTES: usize = 32 * 1024;
const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct Upstream {
    agent: ureq::Agent,
    api_bases: Vec<String>,
}

pub struct PreparedRequest {
    path: String,
    query: Vec<(String, String)>,
}

pub struct UpstreamResponse {
    pub url: String,
    pub status: u16,
    pub content_type: String,
    pub body: Vec<u8>,
    pub cache_status: Option<String>,
    pub cache_age: Option<String>,
}

impl Upstream {
    pub fn new(api_bases: Vec<String>) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(UPSTREAM_TIMEOUT))
            .http_status_as_error(false)
            .max_redirects(0)
            .user_agent(format!("brk-mcp/{}", env!("CARGO_PKG_VERSION")))
            .build();
        Self {
            agent: ureq::Agent::new_with_config(config),
            api_bases,
        }
    }

    pub fn prepare(
        &self,
        operation: &Operation,
        arguments: &Map<String, Value>,
    ) -> Result<PreparedRequest, String> {
        let mut path = operation.http.path.clone();
        let mut query = Vec::new();

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
                        &encode_path_segment(&values[0]),
                    );
                }
                ParameterLocation::Query => {
                    query.extend(
                        values
                            .into_iter()
                            .map(|value| (parameter.name.clone(), value)),
                    );
                }
            }
        }

        if path.contains('{') || path.contains('}') {
            return Err("a required path parameter is missing".to_string());
        }
        let query_bytes = query
            .iter()
            .map(|(name, value)| name.len() + value.len() + 2)
            .sum::<usize>();
        let longest_base = self.api_bases.iter().map(String::len).max().unwrap_or(0);
        if longest_base
            .saturating_add(path.len())
            .saturating_add(query_bytes)
            > MAX_UPSTREAM_URL_BYTES
        {
            return Err(format!(
                "upstream URL exceeds the {MAX_UPSTREAM_URL_BYTES}-byte limit"
            ));
        }

        Ok(PreparedRequest { path, query })
    }

    pub fn fetch(&self, request: PreparedRequest) -> Result<UpstreamResponse, String> {
        let PreparedRequest { path, query } = request;
        let mut last_error = None;
        for api_base in &self.api_bases {
            let url = format!("{api_base}{path}");
            let response = self
                .agent
                .get(&url)
                .query_pairs(query.iter().cloned())
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
        mut response: ureq::http::Response<ureq::Body>,
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

fn encode_path_segment(value: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    #[test]
    fn encodes_path_segments_without_leaving_separators() {
        assert_eq!(encode_path_segment("a/b c"), "a%2Fb%20c");
        assert_eq!(encode_path_segment("ż"), "%C5%BC");
    }

    #[test]
    fn falls_back_to_the_second_base_after_a_transport_failure() {
        let unavailable = TcpListener::bind("127.0.0.1:0").unwrap();
        let unavailable_address = unavailable.local_addr().unwrap();
        drop(unavailable);

        let fallback = TcpListener::bind("127.0.0.1:0").unwrap();
        let fallback_address = fallback.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = fallback.accept().unwrap();
            let mut request = [0_u8; 1024];
            let read = stream.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..read]).starts_with("GET /health HTTP/1.1"));
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}",
                )
                .unwrap();
        });

        let upstream = Upstream::new(vec![
            format!("http://{unavailable_address}"),
            format!("http://{fallback_address}"),
        ]);
        let response = upstream
            .fetch(PreparedRequest {
                path: "/health".to_string(),
                query: Vec::new(),
            })
            .unwrap();

        server.join().unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, br#"{"ok":true}"#);
        assert_eq!(response.url, format!("http://{fallback_address}/health"));
    }
}
