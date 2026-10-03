#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FeeRate;

/// Recommended fee rates in sat/vB
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RecommendedFees {
    /// Fee rate for fastest confirmation (next block)
    pub fastest_fee: FeeRate,
    /// Fee rate for confirmation within ~30 minutes (3 blocks)
    pub half_hour_fee: FeeRate,
    /// Fee rate for confirmation within ~1 hour (6 blocks)
    pub hour_fee: FeeRate,
    /// Fee rate for economical confirmation
    pub economy_fee: FeeRate,
    /// Minimum relay fee rate
    pub minimum_fee: FeeRate,
}
