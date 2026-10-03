use brk_types::PoolSlug;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Mining pool identification for a block
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockPool {
    /// Unique pool identifier
    #[schemars(example = 44)]
    pub id: u8,

    /// Pool name
    #[schemars(example = &"Foundry USA")]
    pub name: String,

    /// URL-friendly pool identifier
    pub slug: PoolSlug,

    /// This block's ordinal among blocks attributed to this pool
    #[schemars(example = 215_000)]
    pub block_number: u64,

    /// Miner name tags found in coinbase scriptsig
    pub miner_names: Option<Vec<String>>,
}
