use bitview_primitives::{BoundedRatio, StoredF64};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock, LazySpotValuePerBlock,
};
use brk_types::Cents;

#[derive(Clone, Traversable)]
pub struct AwakeVecs {
    /// Sum of supply multiplied by wakefulness across a set of UTXO age ranges.
    /// Each age-range contribution is rounded down to whole satoshis.
    pub supply: LazySpotValuePerBlock,
    /// Share of awake supply that is in loss: the sum of supply in loss
    /// multiplied by wakefulness divided by the sum of total supply multiplied
    /// by wakefulness. Returns NaN when the weighted supply is zero.
    #[traversable(wrap = "supply/in_loss", rename = "share")]
    pub supply_in_loss_share: LazyPerBlock<StoredF64, BoundedRatio>,
    /// Sum of creation-date USD value multiplied by wakefulness across a set of
    /// UTXO age ranges. Creation-date value is each unspent output's BTC value
    /// multiplied by Bitcoin's spot price when it was created.
    pub cap: LazyFiatPerBlock<Cents>,
    /// Wakefulness-weighted mean creation price: awake capitalization divided
    /// by awake supply in BTC. Returns zero when awake supply is zero.
    pub price: LazyPriceWithRatioPerBlock,
    /// Creation price weighted by invested value and wakefulness:
    /// sum(weight × creation price² × sats) / sum(weight × creation price × sats).
    /// Uses raw cost-basis moments; returns zero when weighted invested value is zero.
    pub capitalized_price: LazyPriceWithRatioPerBlock,
}
