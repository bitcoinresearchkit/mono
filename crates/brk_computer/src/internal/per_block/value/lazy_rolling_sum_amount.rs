use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Sats};

use crate::internal::{LazyPerBlock, LazyRollingSumFromHeight};

#[derive(Clone, Traversable)]
pub struct LazyRollingSumAmountFromHeight {
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    pub sats: LazyRollingSumFromHeight<Sats>,
    pub usd: LazyPerBlock<Dollars, Cents>,
    pub cents: LazyRollingSumFromHeight<Cents>,
}
