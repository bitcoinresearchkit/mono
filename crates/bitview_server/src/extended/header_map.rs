use axum::http::{
    HeaderMap,
    header::{self, IF_NONE_MATCH},
};

pub trait HeaderMapExtended {
    fn has_etag(&self, etag: &str) -> bool;
    fn insert_etag(&mut self, etag: &str);

    fn insert_cache_control(&mut self, value: &str);
    fn insert_cdn_cache_control(&mut self, value: &str);

    fn insert_content_disposition_attachment(&mut self, filename: &str);

    fn insert_content_type_application_json(&mut self);
    fn insert_content_type_text_csv(&mut self);

    fn insert_vary_accept_encoding(&mut self);

    fn insert_deprecation(&mut self, sunset: &'static str);
}

impl HeaderMapExtended for HeaderMap {
    fn has_etag(&self, etag: &str) -> bool {
        self.get(IF_NONE_MATCH).is_some_and(|v| {
            let raw = v.as_bytes();
            let target = etag.as_bytes();
            raw == b"*"
                || raw
                    .split(|&b| b == b',')
                    .any(|entry| normalize_etag(entry) == target)
        })
    }

    fn insert_etag(&mut self, etag: &str) {
        self.insert(header::ETAG, format!("W/\"{etag}\"").parse().unwrap());
    }

    fn insert_cache_control(&mut self, value: &str) {
        self.insert(header::CACHE_CONTROL, value.parse().unwrap());
    }

    fn insert_cdn_cache_control(&mut self, value: &str) {
        self.insert(
            axum::http::HeaderName::from_static("cdn-cache-control"),
            value.parse().unwrap(),
        );
    }

    fn insert_content_disposition_attachment(&mut self, filename: &str) {
        self.insert(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\"")
                .parse()
                .unwrap(),
        );
    }

    fn insert_content_type_application_json(&mut self) {
        self.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    }

    fn insert_content_type_text_csv(&mut self) {
        self.insert(header::CONTENT_TYPE, "text/csv".parse().unwrap());
    }

    fn insert_vary_accept_encoding(&mut self) {
        self.insert(header::VARY, "Accept-Encoding".parse().unwrap());
    }

    fn insert_deprecation(&mut self, sunset: &'static str) {
        self.insert("Deprecation", "true".parse().unwrap());
        self.insert("Sunset", sunset.parse().unwrap());
    }
}

fn normalize_etag(entry: &[u8]) -> &[u8] {
    let s = entry.trim_ascii();
    let s = s.strip_prefix(b"W/").unwrap_or(s);
    s.strip_prefix(b"\"")
        .and_then(|s| s.strip_suffix(b"\""))
        .unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn map(if_none_match: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(IF_NONE_MATCH, HeaderValue::from_str(if_none_match).unwrap());
        h
    }

    #[test]
    fn matches_weak_strong_wildcard_and_list() {
        assert!(map("W/\"s1-abc\"").has_etag("s1-abc"));
        assert!(map("\"s1-abc\"").has_etag("s1-abc"));
        assert!(map("*").has_etag("anything"));
        assert!(map("W/\"a\", W/\"s1-abc\"").has_etag("s1-abc"));
        assert!(map("  W/\"s1-abc\"  ").has_etag("s1-abc"));
    }

    #[test]
    fn rejects_mismatch_and_missing() {
        assert!(!map("W/\"other\"").has_etag("s1-abc"));
        assert!(!HeaderMap::new().has_etag("s1-abc"));
    }
}
