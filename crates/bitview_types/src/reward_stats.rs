use brk_types::{Height, Sats};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::decimal_string;

/// Block reward statistics over a range of blocks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RewardStats {
    /// First block in the range
    pub start_block: Height,
    /// Last block in the range
    pub end_block: Height,
    /// Total coinbase rewards (subsidy + fees) in sats
    #[serde(with = "decimal_string")]
    #[schemars(with = "String", extend("pattern" = "^[0-9]+$"))]
    pub total_reward: Sats,
    /// Total transaction fees in sats
    #[serde(with = "decimal_string")]
    #[schemars(with = "String", extend("pattern" = "^[0-9]+$"))]
    pub total_fee: Sats,
    /// Total transactions, coinbase included
    #[serde(with = "decimal_string")]
    #[schemars(with = "String", extend("pattern" = "^[0-9]+$"))]
    pub total_tx: u64,
}
