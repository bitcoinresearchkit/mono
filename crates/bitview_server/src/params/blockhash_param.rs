use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::BlockHash;

#[derive(Deserialize, JsonSchema)]
pub struct BlockHashParam {
    pub hash: BlockHash,
}
