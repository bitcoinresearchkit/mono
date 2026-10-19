use bitview_types::BlockTxIndex;
use brk_types::BlockHash;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct BlockHashTxIndex {
    pub hash: BlockHash,

    /// Transaction index within the block (0-based)
    #[schemars(example = 0)]
    pub index: BlockTxIndex,
}
