use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::FeeRate;

/// Fee rate percentiles (min, 10%, 25%, 50%, 75%, 90%, max).
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct FeeRatePercentiles {
    /// Minimum fee rate (sat/vB)
    #[serde(rename = "avgFee_0")]
    min: FeeRate,
    /// 10th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_10")]
    pct10: FeeRate,
    /// 25th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_25")]
    pct25: FeeRate,
    /// Median fee rate (sat/vB)
    #[serde(rename = "avgFee_50")]
    median: FeeRate,
    /// 75th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_75")]
    pct75: FeeRate,
    /// 90th percentile fee rate (sat/vB)
    #[serde(rename = "avgFee_90")]
    pct90: FeeRate,
    /// Maximum fee rate (sat/vB)
    #[serde(rename = "avgFee_100")]
    max: FeeRate,
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
