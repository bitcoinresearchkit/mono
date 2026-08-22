use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_traversable::Traversable;
use brk_types::StoredF64;

use bitview_compute::{LazyColumnPerBlock, LazyPerBlock};

#[derive(Clone, Traversable)]
pub struct ActivitySeries {
    pub wakefulness: AgeRange<LazyColumnPerBlock<StoredF64, AgeRangeId>>,
    /// Dormancy for an exact UTXO age range: one minus wakefulness. Higher
    /// values mean more of the range's accumulated holding time remains stored
    /// rather than consumed by spending.
    pub dormancy: AgeRange<LazyPerBlock<StoredF64>>,
    /// Ratio of consumed to still-stored holding time in an exact UTXO age
    /// range: wakefulness divided by one minus wakefulness. Values above one
    /// mean consumed holding time exceeds stored holding time in the range.
    pub wakefulness_to_dormancy: AgeRange<LazyPerBlock<StoredF64>>,
}
