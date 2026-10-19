use bitview_traversable::Traversable;

use bitview_vecs::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct DormantVecs {
    /// Sum of supply multiplied by one minus wakefulness across the cohort's
    /// UTXO age ranges: the cohort's supply minus its awake supply.
    pub supply: LazySpotValuePerBlock,
}
