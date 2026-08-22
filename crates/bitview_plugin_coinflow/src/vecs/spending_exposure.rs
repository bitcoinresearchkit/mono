use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_traversable::Traversable;
use brk_types::StoredF64;
use derive_more::{Deref, DerefMut};

use bitview_compute::{LazyColumnPerBlock, LazyPerBlock};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct SpendingExposureSeries {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub age_range: AgeRange<LazyColumnPerBlock<StoredF64, AgeRangeId>>,
    /// Estimated probability that supply in a UTXO age range will ever be
    /// spent: one minus exp of negative spending exposure. Nonpositive or NaN
    /// exposure returns zero; positive results are capped just below one. A
    /// value near zero identifies supply unlikely to move, while a value near
    /// one identifies supply likely to move eventually.
    pub mobility: AgeRange<LazyPerBlock<StoredF64>>,
}
