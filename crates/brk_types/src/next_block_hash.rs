use std::fmt;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Content hash of the projected next block (block 0 of the mempool
/// snapshot), including its statistics and complete transaction bodies.
/// Opaque token, distinct from HTTP ETag formatting: pass back
/// to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(transparent)]
pub struct NextBlockHash(u64);

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

impl From<u64> for NextBlockHash {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<NextBlockHash> for u64 {
    fn from(value: NextBlockHash) -> Self {
        value.0
    }
}
