use brk_traversable::Traversable;
use brk_types::StoredF64;

use crate::internal::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct HorizonVecs {
    #[traversable(wrap = "supply/in_loss", rename = "share")]
    pub supply_in_loss_share: LazyPerBlock<StoredF64>,
}
