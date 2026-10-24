use brk_types::FeeRate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Effective fee rate percentiles averaged over a bucket of blocks: each field is the mean of
/// the blocks' own vsize-weighted percentile of their non-coinbase transactions (min and max are
/// mempool.space's effective range ends), in sat/vB as `T` holds it (whole, or to 3 decimals). A
/// block without such transactions counts as 0.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct FeeRatePercentiles<T> {
    /// Mean of the blocks' minimum fee rate (sat/vB)
    #[serde(rename = "avgFee_0")]
    pub min: T,
    /// Mean of the blocks' 10th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_10")]
    pub p10: T,
    /// Mean of the blocks' 25th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_25")]
    pub p25: T,
    /// Mean of the blocks' median fee rate (sat/vB)
    #[serde(rename = "avgFee_50")]
    pub median: T,
    /// Mean of the blocks' 75th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_75")]
    pub p75: T,
    /// Mean of the blocks' 90th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_90")]
    pub p90: T,
    /// Mean of the blocks' maximum fee rate (sat/vB)
    #[serde(rename = "avgFee_100")]
    pub max: T,
}

impl FeeRatePercentiles<FeeRate> {
    /// Each rate rounded to a whole sat/vB, half away from zero, as mempool.space's
    /// `CAST(AVG(..) AS INT)`.
    pub fn whole(self) -> FeeRatePercentiles<u64> {
        let whole = |rate: FeeRate| f64::from(rate).round() as u64;
        FeeRatePercentiles {
            min: whole(self.min),
            p10: whole(self.p10),
            p25: whole(self.p25),
            median: whole(self.median),
            p75: whole(self.p75),
            p90: whole(self.p90),
            max: whole(self.max),
        }
    }
}
