use bitview_primitives::CentsFract;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, PriceWithRatio};
use brk_types::{Cents, Height};
use vecdb::{Rw, StorageMode};

use super::{ema_vecs::EmaVecs, sma::SmaVecs};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Simple moving average of block-level Bitcoin spot prices over a trailing
    /// monotonic-time window, including the represented block. Windows start no earlier
    /// than the first exchange price (block 74,641); null before it.
    #[traversable(lead = "Simple moving average: the price, averaged over the window.")]
    pub sma: SmaVecs<M>,
    /// Exponential moving average of block-level Bitcoin spot price. At each
    /// block it recursively applies `alpha = 2 / (span + 1)`, where `span` is
    /// the number of blocks from the trailing period's monotonic-time start
    /// through the represented block. Windows start no earlier than the first
    /// exchange price (block 74,641), which seeds the average; null before it.
    #[traversable(lead = "Exponential moving average: the price, averaged over the window, recent blocks weighing most.")]
    pub(crate) ema: EmaVecs<PriceWithRatio<M>>,
    #[traversable(hidden)]
    /// Exact exponential-average state in fractional cents; the views round it to whole cents.
    pub(crate) ema_stored: EmaVecs<CachedSeries<Height, CentsFract, M>>,
    /// Cumulative integer-cent prices shared by all SMA windows.
    #[traversable(hidden)]
    pub(crate) sma_prefix_sum: CachedSeries<Height, Cents, M>,
}
