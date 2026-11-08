//! One spelling per answer. A CDN keys on a URL's text, so a respelling that changes nothing
//! would reach the server every time. Refused before any work: a `?` or `&` with nothing to
//! separate, a `%XX` written where it needn't be or in lowercase, and a hash or key that isn't
//! in lowercase. Numbers and lists hold to one spelling in their own types.

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
    if is_canonical(request.uri()) {
        next.run(request).await
    } else {
        Error::bad_request("this URL has another spelling that is the only one accepted")
            .into_response()
    }
}

fn is_canonical(uri: &Uri) -> bool {
    is_canonical_text(uri.path())
        && uri
            .query()
            .is_none_or(|query| !query.split('&').any(str::is_empty) && is_canonical_text(query))
}

fn is_canonical_text(text: &str) -> bool {
    !has_needless_escape(text) && !has_uppercase_hex_key(text)
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
