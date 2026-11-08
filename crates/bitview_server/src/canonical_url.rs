//! One URL per answer: a CDN keys on a URL's text, so spellings that mean the same would each be
//! one more way past its cache.

use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::Error;

/// Refuses a URL with another spelling for the same answer, before anything reads it.
pub async fn reject(request: Request<Body>, next: Next) -> Response {
    let uri = request.uri();
    if is_canonical(uri.path(), uri.query()) {
        next.run(request).await
    } else {
        Error::bad_request(
            "non-canonical URL: empty path segment, empty query or query pair, or a percent-escape \
            of a letter, digit, `-`, `.`, `_` or `~`",
        )
        .into_response()
    }
}

/// Whether the path has no empty segment (a single trailing slash is accepted, and trimmed before
/// routing), the query no empty pair, and neither escapes a character that never needs it.
pub(crate) fn is_canonical(path: &str, query: Option<&str>) -> bool {
    !path.contains("//")
        && !escapes_unreserved(path)
        && query.is_none_or(|query| {
            query.split('&').all(|pair| !pair.is_empty()) && !escapes_unreserved(query)
        })
}

fn escapes_unreserved(text: &str) -> bool {
    let digit = |byte: u8| char::from(byte).to_digit(16);
    text.as_bytes().windows(3).any(|escape| {
        let (b'%', Some(high), Some(low)) = (escape[0], digit(escape[1]), digit(escape[2])) else {
            return false;
        };
        let byte = (high * 16 + low) as u8;
        byte.is_ascii_alphanumeric() || b"-._~".contains(&byte)
    })
}
