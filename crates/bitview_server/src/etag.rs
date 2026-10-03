use axum::http::{
    HeaderMap, HeaderValue,
    header::{ETAG, IF_NONE_MATCH},
};

/// Typed weak entity tag with its validated HTTP representation.
#[derive(Clone, Debug)]
pub struct Etag(HeaderValue);

impl Etag {
    pub fn matches(&self, headers: &HeaderMap) -> bool {
        let target = self.token();
        headers.get_all(IF_NONE_MATCH).iter().any(|value| {
            let value = value.as_bytes().trim_ascii();
            if value == b"*" {
                return true;
            }
            // A whole-field match needs no list parsing. The target was validated
            // at construction, so equality also establishes valid token bytes.
            if Self::normalize(value) == Some(target) {
                return true;
            }
            let mut quoted = false;
            value
                .split(|&byte| {
                    if byte == b'"' {
                        quoted = !quoted;
                    }
                    byte == b',' && !quoted
                })
                .any(|entry| Self::normalize(entry.trim_ascii()) == Some(target))
        })
    }

    pub fn insert(&self, headers: &mut HeaderMap) {
        headers.insert(ETAG, self.0.clone());
    }

    fn token(&self) -> &[u8] {
        let value = self.0.as_bytes();
        &value[3..value.len() - 1]
    }

    fn normalize(value: &[u8]) -> Option<&[u8]> {
        let value = value.strip_prefix(b"W/").unwrap_or(value);
        value.strip_prefix(b"\"")?.strip_suffix(b"\"")
    }
}

impl From<String> for Etag {
    fn from(value: String) -> Self {
        assert!(
            value
                .bytes()
                .all(|byte| byte == 0x21 || (0x23..=0x7e).contains(&byte) || byte >= 0x80),
            "invalid ETag token"
        );
        let mut header = String::with_capacity(value.len() + 4);
        header.push_str("W/\"");
        header.push_str(&value);
        header.push('"');
        Self(HeaderValue::try_from(header).unwrap())
    }
}
