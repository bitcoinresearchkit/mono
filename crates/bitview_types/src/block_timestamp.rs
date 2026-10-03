use brk_types::{BlockHash, Height};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Block information returned for timestamp queries
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockTimestamp {
    /// Block height
    pub height: Height,

    /// Block hash
    pub hash: BlockHash,

    /// Block timestamp in ISO 8601 format
    #[schemars(example = &"2024-04-20T00:00:00.000Z")]
    pub timestamp: String,
}
