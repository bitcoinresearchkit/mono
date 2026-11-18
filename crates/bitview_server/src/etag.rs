use axum::http::{HeaderMap, HeaderValue, header::ETAG};
#[cfg(feature = "full-api")]
use bitview_query::RepresentationId;
#[cfg(feature = "full-api")]
use brk_types::Version;

use crate::VERSION;

/// Typed weak entity tag with its validated HTTP representation. Every tag starts with the
/// release, so a release changing a response's format never validates an older body.
#[derive(Clone, Debug)]
pub struct Etag(HeaderValue);

impl Etag {
    /// Names bytes by their hash: `l{hash:x}`.
    pub fn content(hash: u64) -> Self {
        format!("l{hash:x}").into()
    }

    /// Names a representation the way it identifies itself: by its content, or by the block that
    /// fixes it (`tx3-{version}-{hash}`, `version` bumped when its format changes).
    #[cfg(feature = "full-api")]
    pub fn representation(version: Version, identity: RepresentationId) -> Self {
        match identity {
            RepresentationId::Content(hash) => Self::content(hash),
            RepresentationId::Block(hash) => format!("tx3-{version}-{hash}").into(),
        }
    }

    pub fn matches(&self, headers: &HeaderMap) -> bool {
        bitview_http::if_none_match(headers, Some(self.token()))
    }

    pub fn insert(&self, headers: &mut HeaderMap) {
        headers.insert(ETAG, self.0.clone());
    }

    fn token(&self) -> &[u8] {
        let value = self.0.as_bytes();
        &value[3..value.len() - 1]
    }
}

impl From<String> for Etag {
    fn from(value: String) -> Self {
        debug_assert!(
            value
                .bytes()
                .all(|byte| byte == 0x21 || (0x23..=0x7e).contains(&byte) || byte >= 0x80),
            "invalid ETag token"
        );
        let mut header = String::with_capacity(VERSION.len() + value.len() + 5);
        header.push_str("W/\"");
        header.push_str(&VERSION);
        header.push('-');
        header.push_str(&value);
        header.push('"');
        Self(HeaderValue::try_from(header).unwrap())
    }
}
