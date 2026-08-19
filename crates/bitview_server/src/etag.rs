use std::fmt;

/// Typed entity-tag wrapper. Owns a `String` so values built from `format!`
/// can be passed around without re-allocating, while keeping callsites typed
/// (`Etag` instead of `String`).
#[derive(Clone, Debug)]
pub struct Etag(String);

impl Etag {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Etag {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for Etag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
