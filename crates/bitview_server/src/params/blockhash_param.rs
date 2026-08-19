use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::BlockHash;

/// Block hash path parameter
#[derive(Deserialize, JsonSchema)]
pub struct BlockHashParam {
    pub hash: BlockHash,
}
