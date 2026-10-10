use bitview_traversable::Traversable;

use bitview_vecs::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct LazyBaseVecs {
    /// Circulating supply multiplied by one minus liveliness, where liveliness
    /// is cumulative coinblocks destroyed divided by cumulative coinblocks
    /// created.
    pub vaulted: LazySpotValuePerBlock,
    /// HODLed or lost supply: the vaulted supply under its Glassnode name.
    pub hodled_or_lost: LazySpotValuePerBlock,
    /// Circulating supply multiplied by cumulative coinblocks destroyed divided
    /// by cumulative coinblocks created.
    pub active: LazySpotValuePerBlock,
}
