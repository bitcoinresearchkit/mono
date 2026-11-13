use std::borrow::Cow;

use axum::{
    body::{Body, to_bytes},
    http::{
        Request, Response, StatusCode,
        header::{ALLOW, CONTENT_TYPE},
    },
    middleware::Next,
    response::IntoResponse,
};
use bitview_http::CachePolicy;
use bitview_types::ErrorCode;

use crate::error::{Error, changes_hint};

const MAX_ERROR_BODY_BYTES: usize = 4096;

/// Whether an error body is a document already, left as it is: JSON (`application/json`,
/// `application/...+json`) or a page (`text/html`), ignoring parameters like `; charset=utf-8`.
fn is_document(content_type: &str) -> bool {
    let mime = bitview_http::media_type(content_type);
    mime == "application/json"
        || mime == "text/html"
        || (mime.starts_with("application/") && mime.ends_with("+json"))
}

/// The query parameters v0.11 and v0.12 also read under another name, with the name read now; a
/// parameter renamed later adds its former name here.
const FORMER_PARAMS: &[(&str, &str)] = &[
    ("from", "start"),
    ("f", "start"),
    ("s", "start"),
    ("to", "end"),
    ("t", "end"),
    ("e", "end"),
    ("count", "limit"),
    ("c", "limit"),
    ("l", "limit"),
    ("metrics", "series"),
    ("m", "series"),
    ("i", "index"),
    ("p", "page"),
    ("bucket", "agg"),
];

/// axum's refusal of a query parameter under a former name names its fix in place of the fields
/// it lists, where the route reads the new name; any other text is left as it is. The only place
/// that parses rejection text. It relies on the wording ``Failed to deserialize query string:
/// from: unknown field `from`, expected …`` (axum, serde_path_to_error, serde), read whole so a
/// key can't forge it. A change of that wording silently drops the hint (the old list comes
/// back); the only guard is `cargo api`'s "former query parameter name" error snapshot.
fn with_param_fix(msg: &str) -> Cow<'_, str> {
    const QUERY: &str = "Failed to deserialize query string: ";
    let fix = msg.strip_prefix(QUERY).and_then(|rest| {
        // The list is serde's, of the route's own names; the key before it is the caller's.
        let (head, expected) = rest.rsplit_once("`, expected ")?;
        let &(former, now) = FORMER_PARAMS.iter().find(|&&(former, _)| {
            head.strip_prefix(former)
                .and_then(|head| head.strip_prefix(": unknown field `"))
                == Some(former)
        })?;
        // The names read are the ones in backticks: `a`, `a` or `b`, one of `a`, `b`, ….
        expected
            .split('`')
            .skip(1)
            .step_by(2)
            .any(|name| name == now)
            .then(|| format!("{QUERY}{former}: unknown field `{former}`, use `{now}`"))
    });
    fix.map_or(Cow::Borrowed(msg), Cow::Owned)
}

pub(crate) async fn respond(request: Request<Body>, next: Next) -> Response<Body> {
    let response = next.run(request).await;
    let status = response.status();
    if status.is_success()
        || status.is_redirection()
        || status.is_informational()
        || response
            .headers()
            .get(CONTENT_TYPE)
            .is_some_and(|v| v.to_str().is_ok_and(is_document))
    {
        return response;
    }

    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, MAX_ERROR_BODY_BYTES)
        .await
        .unwrap_or_default();
    let msg = String::from_utf8_lossy(&bytes);
    let (code, msg) = match parts.status {
        StatusCode::NOT_FOUND => (
            ErrorCode::NotFound,
            if msg.is_empty() {
                "Not found".into()
            } else {
                msg
            },
        ),
        StatusCode::METHOD_NOT_ALLOWED => (
            ErrorCode::MethodNotAllowed,
            "Method not allowed for this endpoint".into(),
        ),
        // A request's path, query or body its route couldn't read (axum's rejection): often one written for a release
        // that named things otherwise.
        StatusCode::BAD_REQUEST => (
            ErrorCode::BadRequest,
            format!(
                "{}. {}",
                if msg.is_empty() {
                    "Bad request".into()
                } else {
                    with_param_fix(msg.trim_end_matches('.'))
                },
                changes_hint(None)
            )
            .into(),
        ),
        s if s.is_client_error() => (
            ErrorCode::BadRequest,
            if msg.is_empty() {
                "Bad request".into()
            } else {
                msg
            },
        ),
        _ => (
            ErrorCode::InternalError,
            if msg.is_empty() {
                "Internal server error".into()
            } else {
                msg
            },
        ),
    };
    let msg = msg.into_owned();
    let mut response = Error::new(parts.status, code, msg).into_response();
    // The rewrite changes the body, not the methods allowed or a cache policy its handler marked.
    if let Some(policy) = parts.extensions.get::<CachePolicy>() {
        policy.apply(response.headers_mut());
    }
    if let Some(allow) = parts.headers.get(ALLOW) {
        response.headers_mut().insert(ALLOW, allow.clone());
    }
    response.extensions_mut().extend(parts.extensions);
    response
}
