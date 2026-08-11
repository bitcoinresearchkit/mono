use brk_cohort::{AgeRange, AgeRangeId};
use brk_traversable::Traversable;
use brk_types::StoredF64;
use derive_more::{Deref, DerefMut};

use crate::internal::{LazyColumnPerBlock, LazyPerBlock};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct SpendingExposureSeries {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub age_range: AgeRange<LazyColumnPerBlock<StoredF64, AgeRangeId>>,
    pub mobility: AgeRange<LazyPerBlock<StoredF64>>,
}
