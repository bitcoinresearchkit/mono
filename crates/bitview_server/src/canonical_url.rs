//! One spelling per answer. A CDN keys on a URL's text, so a respelling that changes nothing
//! would reach the server every time. Refused before any work: a `?` or `&` with nothing to
//! separate, a `%XX` written where it needn't be or in lowercase, and a hash or key that isn't
//! in lowercase. Numbers and lists hold to one spelling in their own types.

use std::iter;

use axum::{
    extract::Request,
    http::Uri,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::Error;

/// Hex at least this long is a hash or a public key.
const MIN_HEX_KEY: usize = 64;

pub async fn refuse_respelled(request: Request, next: Next) -> Response {
    match respelling(request.uri()) {
        None => next.run(request).await,
        Some(fix) => Error::bad_request(format!(
            "this URL has another spelling that is the only one accepted: {fix}"
        ))
        .into_response(),
    }
}

/// How to write a URL that isn't in its one spelling.
fn respelling(uri: &Uri) -> Option<&'static str> {
    let query = uri.query();
    let texts = || iter::once(uri.path()).chain(query);
    if query.is_some_and(|query| query.split('&').any(str::is_empty)) {
        Some("remove the `?` or `&` that separates nothing")
    } else if texts().any(has_needless_escape) {
        Some("write `%XX` in uppercase, and only for a character that needs it")
    } else if texts().any(has_uppercase_hex_key) {
        Some("write hashes and keys in lowercase")
    } else {
        None
    }
}

/// `~` isn't counted: browsers' `URLSearchParams` always writes it as `%7E`.
fn has_needless_escape(text: &str) -> bool {
    text.as_bytes().windows(3).any(|window| {
        let [b'%', high, low] = *window else {
            return false;
        };
        let nibble = |digit: u8| (digit as char).to_digit(16).unwrap_or(0) as u8;
        let byte = nibble(high) << 4 | nibble(low);
        high.is_ascii_lowercase()
            || low.is_ascii_lowercase()
            || byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'.' | b'_')
    })
}

fn has_uppercase_hex_key(text: &str) -> bool {
    text.split(['/', '=', '&']).any(|part| {
        part.len() >= MIN_HEX_KEY
            && part.bytes().all(|byte| byte.is_ascii_hexdigit())
            && part.bytes().any(|byte| byte.is_ascii_uppercase())
    })
}
