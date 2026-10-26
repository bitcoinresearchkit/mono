use bitview_traversable::Traversable;

use bitview_vecs::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct DormantVecs {
    /// Sum of supply multiplied by one minus wakefulness across the cohort's
    /// UTXO age ranges: the cohort's supply minus its awake supply.
    #[traversable(lead = "Dormant supply: the coins, each age weighted by how much of its holding time is still stored.")]
    pub supply: LazySpotValuePerBlock,
}
