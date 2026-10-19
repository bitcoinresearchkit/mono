use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BlockSizeEntry, BlockWeightEntry};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BlockSizesWeights {
    pub sizes: Vec<BlockSizeEntry>,
    pub weights: Vec<BlockWeightEntry>,
}
