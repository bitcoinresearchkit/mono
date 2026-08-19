use bitview_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Sats};

use crate::{LazyPerBlock, LazyRollingSumFromHeight};

#[derive(Clone, Traversable)]
pub struct LazyRollingSumAmountFromHeight {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    /// Reported in satoshis.
    pub sats: LazyRollingSumFromHeight<Sats>,
    /// Reported in US dollars.
    pub usd: LazyPerBlock<Dollars, Cents>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: LazyRollingSumFromHeight<Cents>,
}
