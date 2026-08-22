use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use bitview_compute::{LazyPriceWithRatioPerBlock, PriceWithRatioPerBlock};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Realized price divided by one minus liveliness, where liveliness is
    /// cumulative coinblocks destroyed divided by cumulative coinblocks
    /// created. This raises realized price as a larger share of accumulated
    /// holding time is consumed rather than stored.
    pub vaulted: PriceWithRatioPerBlock<M>,
    /// Realized price divided by liveliness, where liveliness is cumulative
    /// coinblocks destroyed divided by cumulative coinblocks created.
    /// This raises realized price when little accumulated holding time has been
    /// consumed.
    pub active: PriceWithRatioPerBlock<M>,
    /// Investor capitalization, equal to realized capitalization minus the
    /// cumulative issuance-date USD value of the derived block-subsidy
    /// component, divided by active supply in BTC. Active supply is circulating
    /// supply multiplied by liveliness.
    pub true_market_mean: PriceWithRatioPerBlock<M>,
    /// Cumulative cointime value destroyed divided by cumulative coinblocks
    /// stored, expressed as a price per BTC. It represents the average value
    /// destroyed for each unit of holding time that remains stored.
    pub cointime: LazyPriceWithRatioPerBlock,
}
