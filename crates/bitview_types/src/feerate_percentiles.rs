use brk_types::FeeRate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Fee rate percentiles (min, 10%, 25%, 50%, 75%, 90%, max).
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct FeeRatePercentiles {
    /// Minimum fee rate (sat/vB)
    #[serde(rename = "avgFee_0")]
    pub min: FeeRate,
    /// 10th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_10")]
    pub pct10: FeeRate,
    /// 25th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_25")]
    pub pct25: FeeRate,
    /// Median fee rate (sat/vB)
    #[serde(rename = "avgFee_50")]
    pub median: FeeRate,
    /// 75th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_75")]
    pub pct75: FeeRate,
    /// 90th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_90")]
    pub pct90: FeeRate,
    /// Maximum fee rate (sat/vB)
    #[serde(rename = "avgFee_100")]
    pub max: FeeRate,
}

impl FeeRatePercentiles {
    pub fn new(
        min: FeeRate,
        pct10: FeeRate,
        pct25: FeeRate,
        median: FeeRate,
        pct75: FeeRate,
        pct90: FeeRate,
        max: FeeRate,
    ) -> Self {
        Self {
            min,
            pct10,
            pct25,
            median,
            pct75,
            pct90,
            max,
        }
    }
}
