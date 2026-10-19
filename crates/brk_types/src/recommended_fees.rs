#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Recommended fee rates in whole sat/vB (`/api/v1/fees/recommended`)
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RecommendedFees {
    /// Fee rate for fastest confirmation (next block)
    pub fastest_fee: u64,
    /// Fee rate for confirmation within ~30 minutes (second projected block)
    pub half_hour_fee: u64,
    /// Fee rate for confirmation within ~1 hour (third projected block)
    pub hour_fee: u64,
    /// Smoothed, fullness-adjusted third-block fee estimate, bounded by `minimumFee` and twice
    /// it; `minimumFee` when no third projected block exists
    pub economy_fee: u64,
    /// The node's current mempool minimum fee rate, rounded up to a whole sat/vB (at least 1)
    pub minimum_fee: u64,
}
