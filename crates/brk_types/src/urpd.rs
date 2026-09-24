use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Bitcoin, Cohort, Date, Dollars, UrpdAggregation, UrpdBucket, UrpdWeight};

/// UTXO Realized Price Distribution for a cohort on a specific date.
///
/// Supply is grouped by the close price at which each UTXO was last moved.
/// Each bucket exposes three values: supply in BTC, realized cap contribution
/// in USD (sum of `realized_price * supply` over the coins in the bucket), and
/// unrealized P&L in USD (`close * supply - realized_cap`, can be negative).
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Urpd {
    pub cohort: Cohort,
    pub date: Date,
    /// Weighting applied to the source supply.
    pub weight: UrpdWeight,
    /// Aggregation strategy applied to the buckets.
    pub aggregation: UrpdAggregation,
    /// Close price on `date`, in USD. Anchor for `unrealized_pnl`.
    pub close: Dollars,
    /// Sum of `supply` across all buckets, in BTC.
    pub total_supply: Bitcoin,
    pub buckets: Vec<UrpdBucket>,
}
