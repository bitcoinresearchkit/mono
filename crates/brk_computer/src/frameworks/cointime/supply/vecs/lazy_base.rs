use brk_traversable::Traversable;

use crate::internal::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct LazyBaseVecs {
    /// Circulating supply multiplied by vaultedness.
    pub vaulted: LazySpotValuePerBlock,
    /// Circulating supply multiplied by liveliness.
    pub active: LazySpotValuePerBlock,
}
