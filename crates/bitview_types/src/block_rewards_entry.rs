use brk_types::{Dollars, Height, Sats, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single block rewards data point.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockRewardsEntry {
    /// Average block height in this window
    #[schemars(example = 890621)]
    pub avg_height: Height,
    /// Unix timestamp at the window midpoint
    #[schemars(example = 1743631892)]
    pub timestamp: Timestamp,
    /// Average coinbase reward per block (subsidy + fees, sats)
    #[schemars(example = 315715861)]
    pub avg_rewards: Sats,
    /// BTC/USD price at this height
    #[serde(rename = "USD")]
    #[schemars(example = 84342.12)]
    pub usd: Dollars,
}
