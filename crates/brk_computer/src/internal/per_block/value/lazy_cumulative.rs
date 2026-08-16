use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Sats};

use crate::internal::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct LazyCumulativeValuePerBlock {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    /// Reported in satoshis.
    pub sats: LazyPerBlock<Sats>,
    /// Reported in US dollars.
    pub usd: LazyPerBlock<Dollars, Cents>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: LazyPerBlock<Cents>,
}
