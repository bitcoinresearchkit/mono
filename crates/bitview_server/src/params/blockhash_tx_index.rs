use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::{BlockHash, BlockTxIndex};

/// Block hash + transaction index path parameters
#[derive(Deserialize, JsonSchema)]
pub struct BlockHashTxIndex {
    /// Bitcoin block hash
    pub hash: BlockHash,

    /// Transaction index within the block (0-based)
    #[schemars(example = 0)]
    pub index: BlockTxIndex,
}
