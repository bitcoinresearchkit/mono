use bitview_compute::{AverageWindow, Retained};
use bitview_primitives::Multiple;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, PriceWithRatio};
use brk_types::Cents;
use vecdb::{Rw, StorageMode};

use super::periods::Periods;

#[derive(Traversable)]
#[traversable(client = "Ema")]
pub(crate) struct SmaVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub periods: Periods<StoredSma<M>>,
    /// Mayer Multiple: Bitcoin spot price divided by its 200-day simple moving
    /// average, computed from cent-precision prices; null before the first exchange
    /// price (block 74,641), approximate while prices are a few cents.
    #[traversable(wrap = "200d", rename = "mayer_multiple")]
    #[traversable(lead = "Mayer Multiple: the price against its 200-day average.")]
    pub mayer_multiple: LazyPerBlock<Multiple>,
}

#[derive(Traversable)]
pub(crate) struct StoredSma<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub price: PriceWithRatio<M>,
    #[traversable(hidden)]
    pub cents: CachedSeries<Cents, M>,
    pub window: M::WriteOnly<Retained<AverageWindow>>,
}
