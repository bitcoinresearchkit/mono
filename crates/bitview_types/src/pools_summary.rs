use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::PoolStats;

/// Mining pools response for a time period
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PoolsSummary {
    /// List of pools sorted by block count descending
    pub pools: Vec<PoolStats>,
    /// Total blocks in the time period
    #[schemars(example = 144)]
    pub block_count: u64,
    /// Estimated network hashrate (H/s)
    #[schemars(example = 700_000_000_000_000_000_000_u128)]
    pub last_estimated_hashrate: u128,
    /// Estimated network hashrate over last 3 days (H/s)
    #[schemars(example = 700_000_000_000_000_000_000_u128)]
    pub last_estimated_hashrate3d: u128,
    /// Estimated network hashrate over last 1 week (H/s)
    #[schemars(example = 700_000_000_000_000_000_000_u128)]
    pub last_estimated_hashrate1w: u128,
}
