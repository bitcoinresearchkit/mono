use bitview_traversable::Traversable;

use bitview_vecs::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
#[traversable(client = "Supply")]
pub struct LazyBaseVecs {
    /// Circulating supply multiplied by one minus liveliness.
    #[traversable(lead = "Vaulted supply: the bitcoin judged unmoved, by how little old coins move.")]
    pub vaulted: LazySpotValuePerBlock,
    /// An alias for vaulted supply, calculated from liveliness measured in coinblocks.
    #[traversable(lead = "HODLed or lost supply is another name for vaulted supply.")]
    pub hodled_or_lost: LazySpotValuePerBlock,
    /// Circulating supply multiplied by liveliness.
    #[traversable(lead = "Active supply: the bitcoin judged in use.")]
    pub active: LazySpotValuePerBlock,
}
