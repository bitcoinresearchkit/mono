use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Sats, StoredF32};

use crate::internal::{LazyPerBlock, LazyRollingAvgFromHeight};

#[derive(Clone, Traversable)]
pub struct LazyRollingAvgAmountFromHeight {
    pub btc: LazyPerBlock<Bitcoin, StoredF32>,
    pub sats: LazyRollingAvgFromHeight<Sats>,
    pub usd: LazyPerBlock<Dollars, StoredF32>,
    pub cents: LazyRollingAvgFromHeight<Cents>,
}
