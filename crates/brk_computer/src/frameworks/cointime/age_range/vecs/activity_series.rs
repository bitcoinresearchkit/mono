use brk_cohort::{AgeRange, AgeRangeId};
use brk_traversable::Traversable;
use brk_types::StoredF64;

use crate::internal::{LazyColumnPerBlock, LazyPerBlock};

#[derive(Clone, Traversable)]
pub struct ActivitySeries {
    pub wakefulness: AgeRange<LazyColumnPerBlock<StoredF64, AgeRangeId>>,
    pub dormancy: AgeRange<LazyPerBlock<StoredF64>>,
    pub wakefulness_to_dormancy: AgeRange<LazyPerBlock<StoredF64>>,
}
