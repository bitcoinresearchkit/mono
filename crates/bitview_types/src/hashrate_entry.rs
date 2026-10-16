use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single hashrate data point.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HashrateEntry {
    /// End of the day the hashrate covers (Unix timestamp), as mempool.space stamps it
    pub timestamp: Timestamp,
    /// Average hashrate (H/s)
    #[serde(rename = "avgHashrate")]
    #[schemars(example = 700_000_000_000_000_000_000_u128)]
    pub avg_hashrate: u128,
}
