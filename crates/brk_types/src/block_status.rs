use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BlockHash, Height};

/// Block status indicating whether block is in the best chain
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockStatus {
    /// Whether this block is in the best chain
    #[schemars(example = true)]
    in_best_chain: bool,

    /// Block height (only if in best chain)
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<Height>,

    /// Hash of the next block in the best chain (null if tip)
    next_best: Option<BlockHash>,
}

impl BlockStatus {
    pub fn in_best_chain(height: Height, next_best: Option<BlockHash>) -> Self {
        Self {
            in_best_chain: true,
            height: Some(height),
            next_best,
        }
    }
}
