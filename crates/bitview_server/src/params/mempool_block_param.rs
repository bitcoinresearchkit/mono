use schemars::JsonSchema;
use serde::Deserialize;

use brk_types::{MempoolBlockHash, MempoolBlockIndex};

#[derive(Deserialize, JsonSchema)]
pub struct MempoolBlockParam {
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub index: MempoolBlockIndex,
}

/// The block and the content token a client holds for it.
#[derive(Deserialize, JsonSchema)]
pub struct MempoolBlockDiffParam {
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub index: MempoolBlockIndex,
    /// The `hash` of the client's copy, from the block or a previous diff
    pub since: MempoolBlockHash,
}
