use bitview_traversable::Traversable;
use brk_cohort::{AgeRange, AgeRangeId};
use brk_types::StoredF64;
use derive_more::{Deref, DerefMut};

use bitview_compute::{LazyColumnPerBlock, LazyPerBlock};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct SpendingExposureSeries {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub age_range: AgeRange<LazyColumnPerBlock<StoredF64, AgeRangeId>>,
    /// Estimated probability that supply in the selected age range will ever
    /// be spent: one minus exp of negative spending exposure. Nonpositive or
    /// NaN exposure returns zero; positive results are capped just below one.
    pub mobility: AgeRange<LazyPerBlock<StoredF64>>,
}
