use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Sats};

use crate::internal::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct LazyCumulativeValuePerBlock {
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    pub sats: LazyPerBlock<Sats>,
    pub usd: LazyPerBlock<Dollars, Cents>,
    pub cents: LazyPerBlock<Cents>,
}
