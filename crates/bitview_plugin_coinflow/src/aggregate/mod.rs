use bitview_primitives::{BoundedRatio, Percent};
use bitview_traversable::Traversable;
use brk_types::Cents;

mod sources;

pub use sources::AggregateSources;

use bitview_vecs::{
    LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock, LazySpotValuePerBlock,
};

use crate::Mobility;

#[derive(Clone, Traversable)]
pub struct AggregateVecs {
    pub supply: Mobility<LazySpotValuePerBlock>,
    /// Share of estimated mobile supply that is in loss: the sum of supply in
    /// loss multiplied by remaining-lifetime spending probability divided by
    /// the sum of total supply multiplied by that probability. Returns NaN
    /// when the weighted supply is zero.
    #[traversable(wrap = "supply/mobile/in_loss", rename = "share")]
    pub supply_in_loss_share: LazyPerBlock<Percent, BoundedRatio>,
    /// Sum of creation-date USD value multiplied by remaining-lifetime spending
    /// probability across a set of UTXO age ranges. Creation-date value is each
    /// unspent output's BTC value multiplied by Bitcoin's spot price when it was
    /// created.
    pub cap: LazyFiatPerBlock<Cents>,
    /// Mobility-weighted mean creation price of supply estimated to move
    /// eventually: coinflow capitalization divided by estimated mobile supply
    /// in BTC. Returns zero when mobile supply is zero.
    pub price: LazyPriceWithRatioPerBlock,
    /// Creation price weighted by invested value and remaining-lifetime spending probability:
    /// sum(weight × creation price² × sats) / sum(weight × creation price × sats).
    /// Uses raw cost-basis moments; returns zero when weighted invested value is zero.
    pub capitalized_price: LazyPriceWithRatioPerBlock,
}
