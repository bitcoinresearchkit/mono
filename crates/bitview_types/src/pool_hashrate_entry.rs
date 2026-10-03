use std::borrow::Cow;

use brk_types::Timestamp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single pool hashrate data point.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PoolHashrateEntry {
    /// Unix timestamp
    pub timestamp: Timestamp,
    /// Average hashrate (H/s)
    #[serde(rename = "avgHashrate")]
    #[schemars(example = 200_000_000_000_000_000_000_u128)]
    pub avg_hashrate: u128,
    /// Pool's share of total network hashrate (0.0 - 1.0)
    #[schemars(example = 0.30)]
    pub share: f64,
    /// Pool name
    #[serde(rename = "poolName")]
    #[schemars(example = &"Foundry USA")]
    pub pool_name: Cow<'static, str>,
}
