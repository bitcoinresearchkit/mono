use bitview_traversable::Traversable;
use brk_types::PartsPerMillionSigned64;
use vecdb::{Rw, StorageMode};

use bitview_compute::{ByDcaCagr, ByLookbackPeriod, LazyPercentPerBlock, StdDevPerBlock, Windows};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Bitcoin spot-price return from the first block in a trailing
    /// monotonic-time window through the represented block: represented-block
    /// price divided by the window's starting price, minus one. Positive values
    /// mean price increased and negative values mean it decreased.
    pub periods: ByLookbackPeriod<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    /// Compound annual growth rate of the Bitcoin spot-price return over the
    /// corresponding whole-year trailing period: `(1 + return)^(1 / years) -
    /// 1`. Positive values are annualized gains and negative values are
    /// annualized losses.
    pub cagr: ByDcaCagr<LazyPercentPerBlock<PartsPerMillionSigned64>>,
    /// Arithmetic mean and population standard deviation of per-block
    /// trailing-24-hour spot-price returns over a trailing
    /// monotonic-time window.
    pub sd_24h: Windows<StdDevPerBlock<M>>,
}
