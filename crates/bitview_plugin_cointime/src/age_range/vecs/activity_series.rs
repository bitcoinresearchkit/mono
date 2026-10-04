use bitview_cohort::AgeRange;
use bitview_primitives::{BoundedRatio, Ratio64};
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct ActivitySeries {
    pub wakefulness: AgeRange<LazyPerBlock<Ratio64, BoundedRatio>>,
    /// Dormancy for an exact UTXO age range: one minus wakefulness. Higher
    /// values mean more of the range's accumulated holding time remains stored
    /// rather than consumed by spending. Uses the exact encoded complement.
    pub dormancy: AgeRange<LazyPerBlock<Ratio64, BoundedRatio>>,
    /// Ratio of consumed to still-stored holding time in an exact UTXO age
    /// range: wakefulness divided by one minus wakefulness. Values above one
    /// mean consumed holding time exceeds stored holding time in the range.
    pub wakefulness_to_dormancy: AgeRange<LazyPerBlock<Ratio64, BoundedRatio>>,
}
