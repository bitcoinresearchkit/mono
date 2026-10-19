use bitview_types::BlockTxIndex;
use brk_types::BlockHash;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct BlockHashStartIndex {
    pub hash: BlockHash,

    /// Starting transaction index within the block (0-based)
    #[schemars(example = 0)]
    pub start_index: BlockTxIndex,
}
