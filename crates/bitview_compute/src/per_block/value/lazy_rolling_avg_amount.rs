use bitview_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Sats, StoredF32};

use crate::{LazyPerBlock, LazyRollingAvgFromHeight};

#[derive(Clone, Traversable)]
pub struct LazyRollingAvgAmountFromHeight {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: LazyPerBlock<Bitcoin, StoredF32>,
    /// Reported in satoshis.
    pub sats: LazyRollingAvgFromHeight<Sats>,
    /// Reported in US dollars.
    pub usd: LazyPerBlock<Dollars, StoredF32>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: LazyRollingAvgFromHeight<Cents>,
}
