use bitview_primitives::Date;
use brk_types::{Bitcoin, BlockHash, Dollars, Height};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Cohort, UrpdAggregation, UrpdBuckets, UrpdWeight};

/// UTXO Realized Price Distribution for a cohort at a specific block.
///
/// Supply is grouped by the price at the block in which each UTXO was last moved; each bucket
/// holds its supply, realized cap and unrealized P&L. The same block header as `UtxoSet`
/// (`height`, `hash`, `date`), then its own fields and its buckets as an object of columns.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Urpd {
    /// Exact published block represented by this distribution.
    pub height: Height,
    /// Hash of block `height`.
    pub hash: BlockHash,
    /// UTC date of the represented block's monotonic timestamp.
    pub date: Date,
    pub cohort: Cohort,
    pub weight: UrpdWeight,
    pub aggregation: UrpdAggregation,
    /// Price at `height`, in USD. Anchor for `unrealized_pnl`.
    pub close: Dollars,
    /// Sum of `buckets.supply`, in BTC.
    pub supply: Bitcoin,
    pub buckets: UrpdBuckets,
}
