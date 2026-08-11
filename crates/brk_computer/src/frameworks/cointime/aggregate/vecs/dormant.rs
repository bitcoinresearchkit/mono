use brk_traversable::Traversable;

use crate::internal::LazySpotValuePerBlock;

#[derive(Clone, Traversable)]
pub struct DormantVecs {
    pub supply: LazySpotValuePerBlock,
}
