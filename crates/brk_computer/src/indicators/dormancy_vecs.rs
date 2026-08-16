use brk_traversable::Traversable;
use brk_types::StoredF32;

use crate::internal::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct DormancyVecs {
    pub supply_adj: LazyPerBlock<StoredF32>,
    pub flow: LazyPerBlock<StoredF32>,
}
