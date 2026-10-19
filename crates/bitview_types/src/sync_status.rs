use brk_types::{Height, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SyncStatus {
    /// Height of the last indexed block
    pub indexed_height: Height,
    /// Height of the last computed block (series)
    pub computed_height: Height,
    /// Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
    /// asking the node
    pub tip_height: Height,
    /// Number of blocks behind the tip
    pub blocks_behind: Height,
    /// Human-readable timestamp of the last indexed block (ISO 8601)
    pub last_indexed_at: String,
    /// Unix timestamp of the last indexed block
    pub last_indexed_at_unix: Timestamp,
}
