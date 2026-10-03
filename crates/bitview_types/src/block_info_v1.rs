use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BlockExtras, BlockInfo};

/// Block information with extras, matching mempool.space /api/v1/blocks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockInfoV1 {
    /// Base block information
    #[serde(flatten)]
    pub info: BlockInfo,

    /// Whether this block has been replaced by a longer chain
    #[serde(default)]
    pub stale: bool,

    /// Extended block data
    pub extras: BlockExtras,
}
