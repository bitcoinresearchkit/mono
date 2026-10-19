#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FeeRate;

/// Recommended fee rates in sat/vB, to the thousandth (`/api/v1/fees/precise`)
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PreciseRecommendedFees {
    /// Fee rate for fastest confirmation (next block)
    pub fastest_fee: FeeRate,
    /// Fee rate for confirmation within ~30 minutes (second projected block)
    pub half_hour_fee: FeeRate,
    /// Fee rate for confirmation within ~1 hour (third projected block)
    pub hour_fee: FeeRate,
    /// Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
    /// it; `minimumFee` when no third projected block exists
    pub economy_fee: FeeRate,
    /// The node's current mempool minimum fee rate, rounded up to 0.001 sat/vB (at least 0.001)
    pub minimum_fee: FeeRate,
}
