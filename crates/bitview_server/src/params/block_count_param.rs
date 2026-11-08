use std::num::NonZero;

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct BlockCountParam {
    /// Number of recent blocks to include
    #[schemars(example = 100)]
    #[serde(deserialize_with = "bitview_types::de_plain")]
    pub block_count: NonZero<usize>,
}
