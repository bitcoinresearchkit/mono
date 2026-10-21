use std::{fmt, ops::Deref};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Hex-encoded data as a JSON string.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(transparent)]
#[cfg_attr(feature = "schemars", schemars(extend("pattern" = "^([0-9a-f]{2})*$")))]
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

/// The encoded text's bytes, not the decoded data.
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
