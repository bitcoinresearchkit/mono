use bitview_primitives::{BoundedRatio, Percent};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlock, LazyPerBlock, LazyPriceWithRatioPerBlock, LazySpotValuePerBlock,
};
use brk_types::Cents;

#[derive(Clone, Traversable)]
pub struct MobileVecs {
    /// Sum of supply multiplied by mobility across the cohort's UTXO age
    /// ranges. Each age-range contribution is rounded down to whole satoshis.
    pub supply: LazySpotValuePerBlock,
    /// Share of mobile supply that is in loss: the sum of supply in loss
    /// multiplied by mobility divided by the sum of total supply multiplied by
    /// mobility. Returns NaN when the weighted supply is zero.
    #[traversable(wrap = "supply/in_loss", rename = "share")]
    pub supply_in_loss_share: LazyPerBlock<Percent, BoundedRatio>,
    /// Sum of creation-date USD value multiplied by mobility across the
    /// cohort's UTXO age ranges: the realized capitalization of the mobile
    /// supply. Creation-date value is each unspent output's BTC value
    /// multiplied by Bitcoin's spot price when it was created.
    pub realized_cap: LazyFiatPerBlock<Cents>,
    /// Mobility-weighted mean creation price: mobile realized capitalization
    /// divided by mobile supply in BTC. Returns zero when mobile supply is zero.
    pub realized_price: LazyPriceWithRatioPerBlock,
    /// Creation price weighted by invested value and mobility:
    /// sum(weight × creation price² × sats) / sum(weight × creation price × sats).
    /// Uses raw cost-basis moments; returns zero when weighted invested value is zero.
    pub capitalized_price: LazyPriceWithRatioPerBlock,
}
