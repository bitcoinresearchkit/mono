use bitview_traversable::Traversable;

use bitview_vecs::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct ImmobileVecs {
    /// Sum of supply multiplied by one minus mobility across the cohort's UTXO
    /// age ranges: the cohort's supply minus its mobile supply.
    pub supply: LazySpotValuePerBlock,
}
