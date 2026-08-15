use brk_traversable::Traversable;
use brk_types::Dollars;

use crate::internal::{FiatType, LazyPerBlock, LazyRollingSumFromHeight};

#[derive(Clone, Traversable)]
pub struct LazyRollingSumFiatFromHeight<C: FiatType> {
    pub usd: LazyPerBlock<Dollars, C>,
    pub cents: LazyRollingSumFromHeight<C>,
}
