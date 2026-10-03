use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single hashrate data point.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HashrateEntry {
    /// Unix timestamp
    pub timestamp: Timestamp,
    /// Average hashrate (H/s)
    #[serde(rename = "avgHashrate")]
    #[schemars(example = 700_000_000_000_000_000_000_u128)]
    pub avg_hashrate: u128,
}
