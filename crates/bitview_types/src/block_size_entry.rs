use brk_types::{Height, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single block size data point.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockSizeEntry {
    /// Average block height in this window
    #[schemars(example = 890621)]
    pub avg_height: Height,
    /// Unix timestamp at the window midpoint
    #[schemars(example = 1743631892)]
    pub timestamp: Timestamp,
    /// Rolling 24h median block size (bytes)
    #[schemars(example = 1580000)]
    pub avg_size: u64,
}
