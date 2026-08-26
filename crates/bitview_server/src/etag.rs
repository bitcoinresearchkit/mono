use std::str;

use axum::http::{
    HeaderMap, HeaderValue,
    header::{ETAG, IF_NONE_MATCH},
};

/// Typed weak entity tag with its validated HTTP representation.
#[derive(Clone, Debug)]
pub struct Etag(HeaderValue);

impl Etag {
    pub fn as_str(&self) -> &str {
        str::from_utf8(self.token()).unwrap()
    }

    pub fn matches(&self, headers: &HeaderMap) -> bool {
        let target = self.token();
        headers.get_all(IF_NONE_MATCH).iter().any(|value| {
            value.as_bytes().split(|&byte| byte == b',').any(|entry| {
                let entry = entry.trim_ascii();
                entry == b"*" || Self::normalize(entry) == target
            })
        })
    }

    pub fn insert(&self, headers: &mut HeaderMap) {
        headers.insert(ETAG, self.0.clone());
    }

    fn token(&self) -> &[u8] {
        let value = self.0.as_bytes();
        &value[3..value.len() - 1]
    }

    fn normalize(value: &[u8]) -> &[u8] {
        let value = value.strip_prefix(b"W/").unwrap_or(value);
        value
            .strip_prefix(b"\"")
            .and_then(|value| value.strip_suffix(b"\""))
            .unwrap_or(value)
    }
}

impl From<String> for Etag {
    fn from(value: String) -> Self {
        let mut header = String::with_capacity(value.len() + 4);
        header.push_str("W/\"");
        header.push_str(&value);
        header.push('"');
        Self(HeaderValue::try_from(header).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(values: &[&'static str]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for value in values {
            headers.append(IF_NONE_MATCH, HeaderValue::from_static(value));
        }
        headers
    }

    #[test]
    fn matches_weak_strong_wildcard_and_list() {
        let etag = Etag::from("s1-abc".to_string());
        assert!(etag.matches(&headers(&["W/\"s1-abc\""])));
        assert!(etag.matches(&headers(&["\"s1-abc\""])));
        assert!(etag.matches(&headers(&["*"])));
        assert!(etag.matches(&headers(&["W/\"a\", W/\"s1-abc\""])));
        assert!(etag.matches(&headers(&["  W/\"s1-abc\"  "])));
    }

    #[test]
    fn checks_every_if_none_match_field() {
        let etag = Etag::from("s1-abc".to_string());
        assert!(etag.matches(&headers(&["W/\"other\"", "W/\"s1-abc\""])));
    }

    #[test]
    fn rejects_mismatch_and_missing() {
        let etag = Etag::from("s1-abc".to_string());
        assert!(!etag.matches(&headers(&["W/\"other\""])));
        assert!(!etag.matches(&HeaderMap::new()));
    }

    #[test]
    fn inserts_exact_weak_header() {
        let etag = Etag::from("s1-abc".to_string());
        let mut headers = HeaderMap::new();
        etag.insert(&mut headers);
        assert_eq!(
            headers.get(ETAG),
            Some(&HeaderValue::from_static("W/\"s1-abc\""))
        );
    }
}
