use bitview_traversable::Traversable;

use bitview_vecs::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
#[traversable(client = "Supply")]
pub struct LazyBaseVecs {
    /// Circulating supply multiplied by one minus liveliness.
    pub vaulted: LazySpotValuePerBlock,
    /// HODLed or lost supply: the vaulted supply under its Glassnode name.
    pub hodled_or_lost: LazySpotValuePerBlock,
    /// Circulating supply multiplied by liveliness.
    pub active: LazySpotValuePerBlock,
}
