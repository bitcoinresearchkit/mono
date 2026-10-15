use derive_more::{From, Into};
use std::fmt;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

/// Content hash of the projected next block (block 0 of the mempool
/// snapshot), including its statistics and complete transaction bodies.
/// Opaque token, distinct from HTTP ETag formatting: pass back
/// to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
/// A decimal string, since JavaScript numbers can't hold every 64-bit value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, From, Into)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(with = "String", example = &"9007199254740993"))]
pub struct NextBlockHash(u64);

impl Serialize for NextBlockHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for NextBlockHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map(Self)
            .map_err(D::Error::custom)
    }
}

impl NextBlockHash {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for NextBlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
