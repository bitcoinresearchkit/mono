use bitview_primitives::Date;
use brk_types::{Bitcoin, Dollars, Height};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Cohort, UrpdAggregation, UrpdBucket, UrpdWeight};

/// UTXO Realized Price Distribution for a cohort at a specific block.
///
/// Supply is grouped by the price at the block in which each UTXO was last moved.
/// Each bucket exposes three values: supply in BTC (weighted by `weight`), realized
/// cap contribution in USD (`price * supply` over the coins in the bucket at their
/// grid creation prices, each within 0.05% of the exact price, rounded to the cent
/// per bucket), and unrealized P&L in USD (`close * supply - realized_cap`, can be
/// negative).
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Urpd {
    pub cohort: Cohort,
    /// UTC date of the represented block.
    pub date: Date,
    /// Exact published block represented by this distribution.
    pub height: Height,
    /// Weighting applied to the source supply.
    pub weight: UrpdWeight,
    /// Aggregation strategy applied to the buckets.
    pub aggregation: UrpdAggregation,
    /// Price at `height`, in USD. Anchor for `unrealized_pnl`.
    pub close: Dollars,
    /// Sum of `supply` across all buckets, in BTC.
    pub total_supply: Bitcoin,
    pub buckets: Vec<UrpdBucket>,
}
