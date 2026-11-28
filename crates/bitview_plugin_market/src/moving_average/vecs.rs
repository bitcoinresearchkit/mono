use bitview_primitives::CentsFract;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, PriceWithRatio};
use vecdb::{Rw, StorageMode};

use super::{
    periods::Periods,
    sma::{PiCycle, SmaVecs},
};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Simple moving average of block-level Bitcoin spot prices over a trailing
    /// monotonic-time window, including the represented block; null until the whole
    /// window lies at or after the first exchange price (block 74,641).
    #[traversable(lead = "Simple moving average: the price, averaged over the window.")]
    pub(crate) sma: SmaVecs<M>,
    /// Philip Swift's Pi cycle top (2019), from two more simple moving averages computed as
    /// those above.
    #[traversable(lead = "Pi cycle top: Philip Swift's top signal from two of the price's averages.")]
    pub(crate) pi_cycle: PiCycle,
    /// Exponential moving average of block-level Bitcoin spot price. At each
    /// block it recursively applies `alpha = 2 / (span + 1)`, where `span` is
    /// the number of blocks from the trailing period's monotonic-time start
    /// through the represented block. Windows start no earlier than the first
    /// exchange price (block 74,641), which seeds the average; null before it.
    #[traversable(
        lead = "Exponential moving average: the price, averaged over the window, recent blocks weighing most."
    )]
    #[traversable(client = "Ema")]
    pub(crate) ema: Periods<PriceWithRatio<M>>,
    #[traversable(hidden)]
    /// Exact exponential-average state in fractional cents; the views round it to whole cents.
    pub(crate) ema_stored: Periods<CachedSeries<CentsFract, M>>,
}
