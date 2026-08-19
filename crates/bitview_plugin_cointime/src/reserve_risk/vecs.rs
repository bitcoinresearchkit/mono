use bitview_traversable::Traversable;
use brk_types::{Height, StoredF64};
use vecdb::{EagerVec, PcoVec, Rw, StorageMode};

use bitview_compute::LazyPerBlock;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Spot price in USD divided by the HODL bank.
    pub value: LazyPerBlock<StoredF64>,
    /// Median per-block value of coin days destroyed over the trailing one-year
    /// timestamp window.
    pub vocdd_median_1y: M::Stored<EagerVec<PcoVec<Height, StoredF64>>>,
    /// Cumulative sum of spot price in USD minus the trailing one-year median
    /// value of coin days destroyed.
    pub hodl_bank: M::Stored<EagerVec<PcoVec<Height, StoredF64>>>,
}
