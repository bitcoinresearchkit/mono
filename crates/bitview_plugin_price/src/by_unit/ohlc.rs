use bitview_traversable::Traversable;
use brk_types::{OHLCCents, OHLCDollars, OHLCSats};

use crate::ohlcs::{LazyOhlcCentsVecs, LazyOhlcVecs};

#[derive(Clone, Traversable)]
pub struct OhlcByUnit {
    /// Reported in USD per BTC.
    pub usd: LazyOhlcVecs<OHLCDollars, OHLCCents>,
    /// Reported in cents per BTC.
    pub cents: LazyOhlcCentsVecs,
    /// Reported in sats per USD: 100,000,000 divided by the price in USD per BTC.
    pub sats: LazyOhlcVecs<OHLCSats, OHLCCents>,
}
