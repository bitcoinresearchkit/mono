use std::{fmt, ops::Deref};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Hex-encoded string. Transparent wrapper over `String`: serializes
/// as a plain JSON string and derefs to `str`, so anywhere `&str` or
/// `AsRef<[u8]>` is expected the `Hex` "just works".
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Hex(String);

impl From<String> for Hex {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<Hex> for String {
    fn from(h: Hex) -> Self {
        h.0
    }
}

impl Deref for Hex {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Hex {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for Hex {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
