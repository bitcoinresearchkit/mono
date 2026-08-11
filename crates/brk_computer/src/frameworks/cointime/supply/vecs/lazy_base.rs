use brk_traversable::Traversable;

use crate::internal::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct LazyBaseVecs {
    pub vaulted: LazySpotValuePerBlock,
    pub active: LazySpotValuePerBlock,
}
