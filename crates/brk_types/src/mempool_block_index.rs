#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A projected mempool block's position, as `/api/v1/fees/mempool-blocks` lists them: 0 is the
/// next block, Bitcoin Core's `getblocktemplate` selection.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(example = 0))]
#[serde(transparent)]
pub struct MempoolBlockIndex(u32);

impl MempoolBlockIndex {
    pub const NEXT: Self = Self(0);

    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for MempoolBlockIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
