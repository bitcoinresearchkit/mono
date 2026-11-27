use bitview_compute::{AverageWindow, ComputeRollingStats, Retained};
use bitview_plugin_mappings::{LookbackVecs, Vecs as MappingsVecs};
use bitview_primitives::Multiple;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyIndexedVec, LazyPerBlock, Price, PriceWithRatio, import_cached, priced_starts,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, Ident, LazyVec, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

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
    pub source: SmaSource<M>,
}

/// A simple moving average's stored cents, and the window that computes them.
#[derive(Traversable)]
pub(crate) struct SmaSource<M: StorageMode = Rw> {
    #[traversable(hidden)]
    pub cents: CachedSeries<Cents, M>,
    window: M::WriteOnly<Retained<AverageWindow>>,
    #[traversable(skip)]
    days: usize,
}

impl SmaSource {
    /// `price_sma_<suffix>`'s cents, over `days`.
    pub(crate) fn import(
        db: &Database,
        suffix: &str,
        days: usize,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let cents = import_cached(db, &format!("price_sma_{suffix}_cents_source"), version, mappings)?;
        Ok(Self { cents, window: Default::default(), days })
    }

    pub(crate) fn compute(
        &mut self,
        from: Height,
        lookback: &LookbackVecs,
        close: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.cents.compute_rolling_average(
            from,
            &priced_starts(lookback.start_vec(self.days)),
            close,
            &mut self.window,
            exit,
        )?;
        Ok(())
    }
}

/// Philip Swift's Pi cycle top (2019): the price's 111-day simple moving average against twice
/// its 350-day one. The averages read the prices up to the block alone.
#[derive(Traversable)]
pub(crate) struct PiCycle<M: StorageMode = Rw> {
    /// The 111-day simple moving average, as the other averages are computed.
    #[traversable(lead = "111-day average: the price, averaged over 111 days.")]
    pub sma_111d: Price<LazyPerBlock<Cents>>,
    #[traversable(hidden)]
    pub source_111d: SmaSource<M>,
    /// The 350-day simple moving average, as the other averages are computed.
    #[traversable(lead = "350-day average: the price, averaged over 350 days.")]
    pub sma_350d: Price<LazyPerBlock<Cents>>,
    #[traversable(hidden)]
    pub source_350d: SmaSource<M>,
    /// Twice the 350-day simple moving average: the line the 111-day one crosses at a top in
    /// Philip Swift's Pi cycle top (2019).
    #[traversable(lead = "Pi cycle top line: twice the price's 350-day average.")]
    pub line: Price<LazyPerBlock<Cents>>,
    /// Philip Swift's Pi cycle top (2019): the 111-day simple moving average divided by twice the
    /// 350-day one. At or above 1, the 111-day average is at or above the line: his top signal.
    /// Null before the first exchange price (block 74,641).
    #[traversable(lead = "Pi cycle top: the price's 111-day average against twice its 350-day one; 1 or more signals a top.")]
    pub top: LazyPerBlock<Multiple>,
}

impl PiCycle {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let source_111d = SmaSource::import(db, "111d", 111, version, mappings)?;
        let source_350d = SmaSource::import(db, "350d", 350, version, mappings)?;
        let sma_111d = Price::from_height_source("price_sma_111d", version, &source_111d.cents, mappings);
        let sma_350d = Price::from_height_source("price_sma_350d", version, &source_350d.cents, mappings);
        let line = Price::from_height_source(
            "pi_cycle_top_line",
            version,
            &LazyVec::new(
                "pi_cycle_top_line_cents_source",
                version,
                source_350d.cents.read_only_boxed_clone(),
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
                &source_111d.cents,
                &source_350d.cents,
                |_, short: Cents, long: Cents| Multiple::from(f64::from(short) / (2.0 * f64::from(long))),
            ),
            mappings,
        );
        Ok(Self { sma_111d, source_111d, sma_350d, source_350d, line, top })
    }

    pub(crate) fn compute(
        &mut self,
        from: Height,
        lookback: &LookbackVecs,
        close: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.source_111d.compute(from, lookback, close, exit)?;
        self.source_350d.compute(from, lookback, close, exit)
    }
}
