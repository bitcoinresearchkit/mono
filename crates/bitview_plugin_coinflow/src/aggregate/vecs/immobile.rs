use bitview_traversable::Traversable;

use bitview_vecs::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct ImmobileVecs {
    /// Sum of supply multiplied by one minus mobility across the cohort's UTXO
    /// age ranges: the cohort's supply minus its mobile supply.
    #[traversable(lead = "Immobile supply: the coins, each age weighted by how unlikely it is to be spent.")]
    pub supply: LazySpotValuePerBlock,
}
