use bitview_traversable::Traversable;
use brk_types::{Cents, StoredF64};

mod sources;

pub use sources::AggregateSources;

use bitview_compute::{
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
    /// For each supported forward horizon, the share of supply likely to move
    /// within that horizon that is in loss at the represented block. Each age
    /// range is weighted by one minus exp of the
    /// negative sum of its observed spending hazards times days across that
    /// horizon. Returns NaN when the weighted supply is zero.
    pub horizon: Horizons<HorizonVecs>,
    /// Sum of creation-date USD value multiplied by remaining-lifetime spending
    /// probability across a set of UTXO age ranges. Creation-date value is each
    /// unspent output's BTC value multiplied by Bitcoin's spot price when it was
    /// created.
    pub cap: LazyFiatPerBlock<Cents>,
    /// Mobility-weighted mean creation price of supply estimated to move
    /// eventually: coinflow capitalization divided by estimated mobile supply
    /// in BTC. Returns zero when mobile supply is zero.
    pub price: LazyPriceWithRatioPerBlock,
}
