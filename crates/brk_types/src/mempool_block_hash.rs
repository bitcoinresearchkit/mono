use derive_more::Into;
use std::fmt;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

/// Content hash of a projected mempool block, including its statistics and complete
/// transaction bodies. Opaque token, distinct from HTTP ETag formatting: pass back to
/// `GET /api/mempool-blocks/{index}/diff/{since}` to follow changes.
/// A decimal string, since JavaScript numbers can't hold every 64-bit value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Into)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(with = "String", extend("pattern" = "^[0-9]+$"), example = &"9007199254740993"))]
pub struct MempoolBlockHash(u64);

impl Serialize for MempoolBlockHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for MempoolBlockHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map(Self)
            .map_err(D::Error::custom)
    }
}

impl MempoolBlockHash {
    /// No copy: the `since` of a diff from nothing.
    pub const ZERO: Self = Self(0);

    /// Mints a block identity, leaving zero reserved for clients with no copy.
    pub const fn new(value: u64) -> Self {
        Self(if value == 0 { 1 } else { value })
    }
}

impl fmt::Display for MempoolBlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
