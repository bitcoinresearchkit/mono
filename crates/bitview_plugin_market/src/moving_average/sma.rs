use bitview_compute::{AverageWindow, Retained};
use bitview_primitives::{Multiple, PriceRatio};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyIndexedVec, LazyPerBlock, Price, PriceWithRatio};
use brk_types::{Cents, Version};
use vecdb::{Ident, LazyVec, ReadableCloneableVec, Rw, StorageMode};

use bitview_plugin_mappings::Vecs as MappingsVecs;

use super::periods::SmaPeriods;

#[derive(Traversable)]
#[traversable(client = "Ema")]
pub(crate) struct SmaVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub periods: SmaPeriods<StoredSma<M>>,
    /// Mayer Multiple: Bitcoin spot price divided by its 200-day simple moving
    /// average, computed from cent-precision prices; null until 200 days of priced
    /// blocks (from the first exchange price, block 74,641) fill its window.
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

/// Philip Swift's Pi cycle top (2019): the price's 111-day simple moving average against twice
/// its 350-day one, both from the simple averages. The averages read the prices up to the block
/// alone.
#[derive(Clone, Traversable)]
pub(crate) struct PiCycle {
    /// The 111-day simple moving average, as the other averages are computed.
    #[traversable(lead = "111-day average: the price, averaged over 111 days.")]
    pub sma_111d: Price<LazyPerBlock<Cents>>,
    /// Spot price divided by the 111-day average, computed from cent-precision prices: null where
    /// the average is.
    #[traversable(lead = "111-day ratio: the price divided by its 111-day average.")]
    pub sma_111d_ratio: LazyPerBlock<Multiple, PriceRatio>,
    /// The 350-day simple moving average, as the other averages are computed.
    #[traversable(lead = "350-day average: the price, averaged over 350 days.")]
    pub sma_350d: Price<LazyPerBlock<Cents>>,
    /// Spot price divided by the 350-day average, computed from cent-precision prices: null where
    /// the average is.
    #[traversable(lead = "350-day ratio: the price divided by its 350-day average.")]
    pub sma_350d_ratio: LazyPerBlock<Multiple, PriceRatio>,
    /// Twice the 350-day simple moving average: the line the 111-day one crosses at a top in
    /// Philip Swift's Pi cycle top (2019). Null until the 350-day window is wholly priced.
    #[traversable(lead = "Pi cycle top line: twice the price's 350-day average.")]
    pub line: Price<LazyPerBlock<Cents>>,
    /// Philip Swift's Pi cycle top (2019): the 111-day simple moving average divided by twice the
    /// 350-day one. At or above 1, the 111-day average is at or above the line: his top signal.
    /// Null until the 350-day window is wholly priced (from the first exchange price, block
    /// 74,641).
    #[traversable(lead = "Pi cycle top: the price's 111-day average against twice its 350-day one; 1 or more signals a top.")]
    pub top: LazyPerBlock<Multiple>,
}

impl PiCycle {
    pub(crate) fn new(
        version: Version,
        periods: &SmaPeriods<StoredSma>,
        mappings: &MappingsVecs,
    ) -> Self {
        let (short, long) = (&periods._111d.cents, &periods._350d.cents);
        let line = Price::from_height_source(
            "pi_cycle_top_line",
            version,
            &LazyVec::new(
                "pi_cycle_top_line_cents_source",
                version,
                long.read_only_boxed_clone(),
                |_, cents: Cents| cents * 2,
            ),
            mappings,
        );
        let top = LazyPerBlock::from_height_source::<Ident>(
            "pi_cycle_top",
            version,
            &LazyIndexedVec::new(
                "pi_cycle_top_source",
                version,
                short,
                long,
                |_, short: Cents, long: Cents| Multiple::from(f64::from(short) / (2.0 * f64::from(long))),
            ),
            mappings,
        );
        let (short, long) = (&periods._111d.price, &periods._350d.price);
        Self {
            sma_111d: Price::clone(short),
            sma_111d_ratio: short.relative.ratio.clone(),
            sma_350d: Price::clone(long),
            sma_350d_ratio: long.relative.ratio.clone(),
            line,
            top,
        }
    }
}
