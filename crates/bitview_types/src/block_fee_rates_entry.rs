use brk_types::{FeeRate, Height, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FeeRatePercentiles;

/// One time bucket of blocks. Each `avgFee_*` field is the bucket's mean of its blocks' own
/// percentile of their non-coinbase transactions' effective fee rates, weighted by vsize (a block
/// without such transactions counts as 0), rounded to a whole sat/vB.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockFeeRatesEntry {
    /// Average block height in this bucket
    pub avg_height: Height,
    /// Average block timestamp in this bucket
    pub timestamp: Timestamp,
    #[serde(flatten)]
    pub percentiles: FeeRatePercentiles<u64>,
}

/// One time bucket of blocks, as [`BlockFeeRatesEntry`] with each `avgFee_*` field to 3
/// decimals.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockFeeRatesPreciseEntry {
    /// Average block height in this bucket
    pub avg_height: Height,
    /// Average block timestamp in this bucket
    pub timestamp: Timestamp,
    #[serde(flatten)]
    pub percentiles: FeeRatePercentiles<FeeRate>,
}
