use bitview_primitives::Float64;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;
use brk_types::Height;
use vecdb::{Budgeted, EagerVec, PcoVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Reserve Risk: spot price in USD divided by the HODL bank. Lower positive
    /// values mean spot is cheaper relative to the accumulated holder reserve;
    /// higher values mean it is more expensive.
    pub value: LazyPerBlock<Float64>,
    /// Median over the blocks of the trailing 30 days of the trailing 24-hour
    /// supply-adjusted value of coin days destroyed (VOCDD): each block's coin
    /// days destroyed times its spot price, divided by circulating supply,
    /// summed over the trailing 24 hours, in USD-days per BTC.
    pub vocdd_median_1m: M::Stored<EagerVec<PcoVec<Height, Float64>>>,
    /// HODL bank: running sum of spot price in USD minus the 30-day VOCDD
    /// median, each block weighted by the time since the previous block in days
    /// (on monotonic timestamps), so every day adds one daily step. It
    /// represents the model's accumulated holder reserve and is the denominator
    /// of Reserve Risk.
    pub hodl_bank: M::Stored<EagerVec<PcoVec<Height, Float64, Budgeted>>>,
}
