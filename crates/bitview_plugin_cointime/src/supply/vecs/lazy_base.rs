use bitview_traversable::Traversable;

use bitview_compute::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct LazyBaseVecs {
    /// Circulating supply multiplied by one minus liveliness, where liveliness
    /// is cumulative coinblocks destroyed divided by cumulative coinblocks
    /// created.
    pub vaulted: LazySpotValuePerBlock,
    /// Circulating supply multiplied by cumulative coinblocks destroyed divided
    /// by cumulative coinblocks created.
    pub active: LazySpotValuePerBlock,
}
