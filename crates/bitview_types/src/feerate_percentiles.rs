use brk_types::FeeRate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Effective fee rate percentiles averaged over a bucket of blocks: each field is the mean of
/// the blocks' own vsize-weighted percentile of their non-coinbase transactions (min and max are
/// mempool.space's effective range ends). A block without such transactions counts as 0.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct FeeRatePercentiles {
    /// Mean of the blocks' minimum fee rate (sat/vB)
    #[serde(rename = "avgFee_0")]
    pub min: FeeRate,
    /// Mean of the blocks' 10th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_10")]
    pub p10: FeeRate,
    /// Mean of the blocks' 25th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_25")]
    pub p25: FeeRate,
    /// Mean of the blocks' median fee rate (sat/vB)
    #[serde(rename = "avgFee_50")]
    pub median: FeeRate,
    /// Mean of the blocks' 75th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_75")]
    pub p75: FeeRate,
    /// Mean of the blocks' 90th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_90")]
    pub p90: FeeRate,
    /// Mean of the blocks' maximum fee rate (sat/vB)
    #[serde(rename = "avgFee_100")]
    pub max: FeeRate,
}
