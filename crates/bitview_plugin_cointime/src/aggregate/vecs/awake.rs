use bitview_primitives::{BoundedRatio, Percent};
use bitview_traversable::Traversable;
use bitview_urpd::CostBasisVecs;
use bitview_vecs::{LazyFiatPerBlock, LazyPerBlock, LazySpotValuePerBlock};
use brk_types::Cents;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct AwakeVecs<M: StorageMode = Rw> {
    /// Sum of supply multiplied by wakefulness across the cohort's UTXO age
    /// ranges. Each age-range contribution is rounded down to whole satoshis.
    #[traversable(wrap = "supply", rename = "total")]
    pub supply: LazySpotValuePerBlock,
    /// Share of awake supply that is in loss: the sum of supply in loss
    /// multiplied by wakefulness divided by the sum of total supply multiplied
    /// by wakefulness. Null when the weighted supply is zero.
    #[traversable(wrap = "supply/in_loss", rename = "share")]
    pub supply_in_loss_share: LazyPerBlock<Percent, BoundedRatio>,
    /// Sum of creation-date USD value multiplied by wakefulness across the
    /// cohort's UTXO age ranges: the realized capitalization of the awake
    /// supply. Creation-date value is each unspent output's BTC value
    /// multiplied by Bitcoin's spot price when it was created.
    #[traversable(wrap = "capital", rename = "total")]
    pub capital: LazyFiatPerBlock<Cents>,
    /// Realized cap: the awake capital, under its jargon name.
    #[traversable(wrap = "capital", rename = "realized_cap")]
    pub realized_cap: LazyFiatPerBlock<Cents>,
    /// Creation-price statistics of the awake supply: each unspent output weighted by its
    /// age range's wakefulness.
    pub cost_basis: CostBasisVecs<M>,
}
