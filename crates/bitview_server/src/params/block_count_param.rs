use schemars::JsonSchema;
use serde::Deserialize;

/// Block count path parameter
#[derive(Deserialize, JsonSchema)]
pub struct BlockCountParam {
    /// Number of recent blocks to include
    #[schemars(example = 100)]
    pub block_count: usize,
}
