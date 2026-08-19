use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::NextBlockHash;

/// Prior-template hash for `GET /api/v1/mempool/block-template/diff/{hash}`.
#[derive(Deserialize, JsonSchema)]
pub struct NextBlockHashParam {
    pub hash: NextBlockHash,
}
