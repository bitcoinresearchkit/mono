use bitview_traversable::Traversable;

use bitview_vecs::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct DormantVecs {
    /// Sum of supply multiplied by one minus wakefulness across the cohort's
    /// UTXO age ranges. Each age-range contribution is rounded down to whole
    /// satoshis.
    pub supply: LazySpotValuePerBlock,
}
