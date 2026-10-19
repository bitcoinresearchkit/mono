use brk_types::{Height, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FeeRatePercentiles;

/// One time bucket of blocks. Each `avgFee_*` field is the
/// bucket's mean of its blocks' own percentile of their non-coinbase transactions' effective fee
/// rates, weighted by vsize (a block without such transactions counts as 0).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BlockFeeRatesEntry {
    /// Average block height in this bucket
    pub avg_height: Height,
    /// Average block timestamp in this bucket
    pub timestamp: Timestamp,
    #[serde(flatten)]
    pub percentiles: FeeRatePercentiles,
}
