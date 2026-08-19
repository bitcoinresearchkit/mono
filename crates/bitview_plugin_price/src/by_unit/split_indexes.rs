use bitview_traversable::Traversable;
use brk_types::{Cents, Dollars, OHLCCents, Sats};

use bitview_compute::LazyIndexes;

#[derive(Clone, Traversable)]
pub struct SplitIndexesByUnit {
    /// Reported in USD per BTC.
    pub usd: LazyIndexes<Dollars, Cents>,
    /// Reported in cents per BTC.
    pub cents: LazyIndexes<Cents, OHLCCents>,
    /// Reported in sats per USD: 100,000,000 divided by the price in USD per BTC.
    pub sats: LazyIndexes<Sats, Cents>,
}
