use bitview_traversable::Traversable;

use bitview_compute::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct LazyBaseVecs {
    /// Circulating supply multiplied by vaultedness.
    pub vaulted: LazySpotValuePerBlock,
    /// Circulating supply multiplied by liveliness.
    pub active: LazySpotValuePerBlock,
}
