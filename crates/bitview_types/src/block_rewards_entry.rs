use brk_types::{Dollars, Height, Sats, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single block rewards data point: one time bucket of blocks.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockRewardsEntry {
    /// Average block height in this bucket
    #[schemars(example = 890621)]
    pub avg_height: Height,
    /// Average block timestamp in this bucket
    #[schemars(example = 1743631892)]
    pub timestamp: Timestamp,
    /// Mean coinbase reward (subsidy + fees) per block in this bucket, rounded half up (sats)
    #[schemars(example = 315715861)]
    pub avg_rewards: Sats,
    /// Mean BTC/USD spot price over the bucket's blocks (the price, not the rewards in USD)
    #[serde(rename = "USD")]
    #[schemars(example = 84342.12)]
    pub usd: Dollars,
}
