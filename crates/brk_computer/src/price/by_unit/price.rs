use brk_traversable::Traversable;
use brk_types::{Cents, Dollars, Sats};
use vecdb::{Rw, StorageMode};

use crate::internal::{CachedPerBlock, LazyPerBlock};

#[derive(Traversable)]
pub struct PriceByUnit<M: StorageMode = Rw> {
    /// Reported in USD per BTC.
    pub usd: LazyPerBlock<Dollars, Cents>,
    /// Reported in cents per BTC.
    pub cents: CachedPerBlock<Cents, M>,
    /// Reported in sats per USD: 100,000,000 divided by the price in USD per BTC.
    pub sats: LazyPerBlock<Sats, Cents>,
}
