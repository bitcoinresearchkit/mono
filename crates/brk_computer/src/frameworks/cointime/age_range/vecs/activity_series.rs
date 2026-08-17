use brk_cohort::{AgeRange, AgeRangeId};
use brk_traversable::Traversable;
use brk_types::StoredF64;

use crate::internal::{LazyColumnPerBlock, LazyPerBlock};

#[derive(Clone, Traversable)]
pub struct ActivitySeries {
    pub wakefulness: AgeRange<LazyColumnPerBlock<StoredF64, AgeRangeId>>,
    /// One minus wakefulness for the selected age range.
    pub dormancy: AgeRange<LazyPerBlock<StoredF64>>,
    /// Wakefulness divided by dormancy for the selected age range.
    pub wakefulness_to_dormancy: AgeRange<LazyPerBlock<StoredF64>>,
}
