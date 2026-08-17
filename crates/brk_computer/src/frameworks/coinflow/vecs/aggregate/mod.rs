use brk_traversable::Traversable;
use brk_types::{Cents, StoredF64};

mod sources;

pub use sources::AggregateSources;

use crate::internal::{
    LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock, LazySpotValuePerBlock,
};

use super::{super::Horizons, HorizonVecs, Mobility};

#[derive(Clone, Traversable)]
pub struct AggregateVecs {
    pub supply: Mobility<LazySpotValuePerBlock>,
    /// Share of estimated mobile supply that is in loss: the sum of supply in
    /// loss multiplied by remaining-lifetime spending probability divided by
    /// the sum of total supply multiplied by that probability. Returns NaN
    /// when the weighted supply is zero.
    #[traversable(wrap = "supply/mobile/in_loss", rename = "share")]
    pub supply_in_loss_share: LazyPerBlock<StoredF64>,
    /// Share of supply likely to move within the named forward horizon that is
    /// currently in loss. Each age range is weighted by one minus exp of the
    /// negative sum of its observed spending hazards times days across that
    /// horizon. Returns NaN when the weighted supply is zero.
    pub horizon: Horizons<HorizonVecs>,
    /// Sum of realized capitalization multiplied by remaining-lifetime
    /// spending probability across the selected UTXO age ranges.
    pub cap: LazyFiatPerBlock<Cents>,
    /// Coinflow capitalization divided by estimated mobile supply in BTC.
    /// Returns zero when mobile supply is zero.
    pub price: LazyPriceWithRatioPerBlock,
}
