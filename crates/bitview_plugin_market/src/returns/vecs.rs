use bitview_traversable::Traversable;
use brk_types::PartsPerMillionSigned64;
use vecdb::{Rw, StorageMode};

use bitview_compute::{ByDcaCagr, ByLookbackPeriod, LazyPercentPerBlock, StdDevPerBlock, Windows};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Bitcoin spot-price return from the first block in the named trailing
    /// monotonic-time window through the current block: current price divided
    /// by past price, minus one.
    pub periods: ByLookbackPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    /// Compound annual growth rate of the Bitcoin spot-price return over the
    /// named whole-year trailing period: `(1 + return)^(1 / years) - 1`.
    pub cagr: ByDcaCagr<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    /// Arithmetic mean and population standard deviation of per-block
    /// trailing-24-hour spot-price returns over the named trailing
    /// monotonic-time window.
    pub sd_24h: Windows<StdDevPerBlock<M>>,
}
