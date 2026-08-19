use bitview_traversable::Traversable;
use brk_types::Dollars;

use crate::{FiatType, LazyPerBlock, LazyRollingSumFromHeight};

#[derive(Clone, Traversable)]
pub struct LazyRollingSumFiatFromHeight<C: FiatType> {
    /// Reported in US dollars.
    pub usd: LazyPerBlock<Dollars, C>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: LazyRollingSumFromHeight<C>,
}
