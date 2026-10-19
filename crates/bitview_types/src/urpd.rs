use bitview_primitives::Date;
use brk_types::{Bitcoin, Dollars, Height};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Cohort, UrpdAggregation, UrpdBucket, UrpdWeight};

/// UTXO Realized Price Distribution for a cohort at a specific block.
///
/// Supply is grouped by the price at the block in which each UTXO was last moved; each bucket
/// holds its supply, realized cap and unrealized P&L.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Urpd {
    pub cohort: Cohort,
    /// UTC date of the represented block's monotonic timestamp.
    pub date: Date,
    /// Exact published block represented by this distribution.
    pub height: Height,
    pub weight: UrpdWeight,
    pub aggregation: UrpdAggregation,
    /// Price at `height`, in USD. Anchor for `unrealized_pnl`.
    pub close: Dollars,
    /// Sum of `supply` across all buckets, in BTC.
    pub total_supply: Bitcoin,
    pub buckets: Vec<UrpdBucket>,
}
