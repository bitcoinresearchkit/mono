use bitview_traversable::Traversable;

use bitview_compute::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct DormantVecs {
    /// Sum of supply multiplied by one minus wakefulness across a set of UTXO
    /// age ranges. Each age-range contribution is rounded down to whole
    /// satoshis.
    pub supply: LazySpotValuePerBlock,
}
