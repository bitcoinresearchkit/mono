use brk_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use crate::internal::{LazyPriceWithRatioPerBlock, PriceWithRatioPerBlock};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Realized price divided by vaultedness.
    pub vaulted: PriceWithRatioPerBlock<M>,
    /// Realized price divided by liveliness.
    pub active: PriceWithRatioPerBlock<M>,
    /// Investor capitalization divided by active supply in BTC.
    pub true_market_mean: PriceWithRatioPerBlock<M>,
    /// Cointime capitalization divided by circulating supply in BTC.
    pub cointime: LazyPriceWithRatioPerBlock,
}
