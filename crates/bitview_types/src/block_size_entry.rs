use brk_types::{Height, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One time bucket of blocks.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockSizeEntry {
    /// Average block height in this bucket
    #[schemars(example = 890621)]
    pub avg_height: Height,
    /// Average block timestamp in this bucket
    #[schemars(example = 1743631892)]
    pub timestamp: Timestamp,
    /// Mean block size in this bucket, rounded half up (bytes)
    #[schemars(example = 1580000)]
    pub avg_size: u64,
}
