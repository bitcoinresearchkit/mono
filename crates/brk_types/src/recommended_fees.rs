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
    /// Fee rate for confirmation within ~30 minutes (3 blocks)
    pub half_hour_fee: u64,
    /// Fee rate for confirmation within ~1 hour (6 blocks)
    pub hour_fee: u64,
    /// Fee rate for economical confirmation
    pub economy_fee: u64,
    /// Minimum relay fee rate
    pub minimum_fee: u64,
}
