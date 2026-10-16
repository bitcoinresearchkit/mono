use axum::http::{HeaderMap, HeaderValue, header};

pub trait HeaderMapExtended {
    fn insert_etag(&mut self, etag: &str);
    fn insert_vary_accept_encoding(&mut self);
    fn insert_cache_control_must_revalidate(&mut self);
    fn insert_cache_control_immutable(&mut self);
}

impl HeaderMapExtended for HeaderMap {
    /// The weak tag of an opaque `etag`: compression serves other bytes under the same validator.
    fn insert_etag(&mut self, etag: &str) {
        self.insert(header::ETAG, format!("W/\"{etag}\"").parse().unwrap());
    }

    fn insert_vary_accept_encoding(&mut self) {
        self.insert(header::VARY, HeaderValue::from_static("accept-encoding"));
    }

    fn insert_cache_control_must_revalidate(&mut self) {
        self.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=1, must-revalidate"),
        );
    }

    fn insert_cache_control_immutable(&mut self) {
        self.insert(
            header::CACHE_CONTROL,
            "public, max-age=31536000, immutable".parse().unwrap(),
        );
    }
}
