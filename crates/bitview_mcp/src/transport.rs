//! Streamable HTTP for a server with nothing to stream: one POST per message, answered as `application/json`, and the
//! GET stream refused.

use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Request, State},
    http::{
        HeaderMap, HeaderValue, Method, StatusCode,
        header::{ACCEPT, ALLOW, CACHE_CONTROL, CONTENT_TYPE, VARY},
    },
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use serde_json::json;

use crate::{
    protocol::{self, Answer, SUPPORTED_PROTOCOL_VERSIONS},
    server::AppState,
};

const PROTOCOL_VERSION_HEADER: &str = "mcp-protocol-version";
const JSON: &str = "application/json";
const EVENT_STREAM: &str = "text/event-stream";
const MAX_REQUEST_BYTES: usize = 64 * 1024;

/// `POST /`: one JSON-RPC message, answered in full.
pub async fn post(State(state): State<Arc<AppState>>, request: Request) -> Response {
    let (parts, mut body) = request.into_parts();
    let headers = &parts.headers;
    // Only a version named outright is refused: a request naming none is served as the floor (Gemini CLI sends none).
    let version = match headers.get(PROTOCOL_VERSION_HEADER) {
        None => None,
        Some(header) => match SUPPORTED_PROTOCOL_VERSIONS.into_iter().find(|version| version.as_bytes() == header.as_bytes()) {
            Some(version) => Some(version),
            None => return unsupported_version(header),
        },
    };
    if !(accepts(headers, JSON) && accepts(headers, EVENT_STREAM)) {
        return plain(StatusCode::NOT_ACCEPTABLE, "Not Acceptable: Client must accept both application/json and text/event-stream");
    }
    let json_body = headers
        .get(CONTENT_TYPE)
        .and_then(|header| header.to_str().ok())
        .is_some_and(|header| header.split(';').next().unwrap().trim().eq_ignore_ascii_case(JSON));
    if !json_body {
        return plain(StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported Media Type: Content-Type must be application/json");
    }

    // Counted as it arrives, whatever the framing says.
    let mut bytes = Vec::new();
    while let Some(frame) = body.frame().await {
        let data = match frame {
            Ok(frame) => frame.into_data().unwrap_or_default(),
            Err(error) => return plain(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read request body: {error}")),
        };
        if data.len() > MAX_REQUEST_BYTES - bytes.len() {
            return plain(StatusCode::PAYLOAD_TOO_LARGE, format!("Payload Too Large: request body exceeds {MAX_REQUEST_BYTES} bytes"));
        }
        bytes.extend_from_slice(&data);
    }

    let Answer { status, body } = protocol::answer(&state, headers, version, &bytes).await;
    let response = Response::builder().status(status);
    match body {
        Some(body) => response.header(CONTENT_TYPE, JSON).body(Body::from(body)),
        None => response.body(Body::empty()),
    }
    .expect("a status and a content type make a response")
}

/// `GET /` (and its `HEAD`) is the page. As an event stream it is MCP's GET stream, which this server has not.
pub async fn page_guard(request: Request, next: Next) -> Response {
    let landing = matches!(*request.method(), Method::GET | Method::HEAD) && request.uri().path() == "/";
    if landing && accepts(request.headers(), EVENT_STREAM) {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            [(CACHE_CONTROL, "no-store"), (ALLOW, "GET, HEAD, POST"), (VARY, "Accept")],
        )
            .into_response();
    }
    let mut response = next.run(request).await;
    if landing && !varies_by_accept(response.headers()) {
        response.headers_mut().append(VARY, HeaderValue::from_static("Accept"));
    }
    response
}

/// A plain-text refusal, without a content type.
fn plain(status: StatusCode, message: impl Into<Body>) -> Response {
    Response::builder()
        .status(status)
        .body(message.into())
        .expect("a status makes a response")
}

fn unsupported_version(version: &HeaderValue) -> Response {
    let error = json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": {
            "code": -32022,
            "message": "Unsupported protocol version",
            "data": {
                "requested": String::from_utf8_lossy(version.as_bytes()),
                "supported": SUPPORTED_PROTOCOL_VERSIONS,
            },
        },
    });
    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

/// Whether a response already says it varies by `Accept` (any `Vary` header naming it, any case).
fn varies_by_accept(headers: &HeaderMap) -> bool {
    headers
        .get_all(VARY)
        .iter()
        .filter_map(|vary| vary.to_str().ok())
        .flat_map(|vary| vary.split(','))
        .any(|name| name.trim().eq_ignore_ascii_case("accept"))
}

/// Whether any `Accept` header names `media` (any case, its parameters aside), not refused with a zero weight.
fn accepts(headers: &HeaderMap, media: &str) -> bool {
    headers
        .get_all(ACCEPT)
        .iter()
        .filter_map(|accept| accept.to_str().ok())
        .flat_map(|accept| split_unquoted(accept, ','))
        .any(|range| {
            let mut parts = split_unquoted(range, ';');
            parts.next().is_some_and(|essence| essence.trim().eq_ignore_ascii_case(media))
                && parts.all(|parameter| {
                    parameter.split_once('=').is_none_or(|(name, weight)| {
                        !name.trim().eq_ignore_ascii_case("q") || weight_accepts(weight.trim())
                    })
                })
        })
}

/// RFC 9110 §12.4.2 allows three decimals. A weight outside that grammar is ignored rather than refusing the range, so
/// only an explicit zero refuses it.
fn weight_accepts(weight: &str) -> bool {
    let (whole, fraction) = weight.split_once('.').unwrap_or((weight, ""));
    if fraction.len() > 3 || !fraction.bytes().all(|digit| digit.is_ascii_digit()) {
        return true;
    }
    whole != "0" || fraction.bytes().any(|digit| digit != b'0')
}

/// A header's text split at `delimiter` outside its quoted strings (`\` escaping within them), as RFC 9110 reads a
/// list and its parameters.
fn split_unquoted(text: &str, delimiter: char) -> impl Iterator<Item = &str> {
    let (mut quoted, mut escaped) = (false, false);
    text.split(move |character| {
        if escaped {
            escaped = false;
        } else if quoted && character == '\\' {
            escaped = true;
        } else if character == '"' {
            quoted = !quoted;
        } else if !quoted && character == delimiter {
            return true;
        }
        false
    })
}
