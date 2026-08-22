use bitview_traversable::Traversable;
use brk_types::{Height, StoredF64};
use vecdb::{EagerVec, PcoVec, Rw, StorageMode};

use bitview_compute::LazyPerBlock;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Reserve Risk: spot price in USD divided by the HODL bank, which is the
    /// cumulative sum of spot price minus the trailing-one-year median
    /// supply-adjusted value of coin days destroyed. Lower positive values mean
    /// spot is cheaper relative to this accumulated holder reserve; higher
    /// values mean it is more expensive.
    pub value: LazyPerBlock<StoredF64>,
    /// Median per-block supply-adjusted value of coin days destroyed over the
    /// trailing 365-day timestamp window. Each block's value is spot price
    /// multiplied by coin days destroyed and divided by circulating supply,
    /// producing USD-days per BTC of circulating supply.
    pub vocdd_median_1y: M::Stored<EagerVec<PcoVec<Height, StoredF64>>>,
    /// HODL bank: cumulative sum, through the represented block, of spot price
    /// in USD minus the trailing-365-day median supply-adjusted value of coin
    /// days destroyed. It represents the model's accumulated holder reserve and
    /// is the denominator of Reserve Risk.
    pub hodl_bank: M::Stored<EagerVec<PcoVec<Height, StoredF64>>>,
}
