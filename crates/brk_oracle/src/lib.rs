//! BTC/USD price oracle.
//!
//! Detects round-dollar transaction patterns ($1, $5, $10, ... $10,000) in Bitcoin
//! block outputs to derive the price block by block, starting from a seed price;
//! bundled exchange prices cover the heights before [`START_HEIGHT_SLOW`].
//!
//! Behavior changes by height along two independent axes, each in its own module:
//!
//! - EMA regime (`config`): below [`START_HEIGHT_SLOW`] prices come from the baked
//!   pre-oracle tape. From there to [`START_HEIGHT_FAST`] a slow cold-start EMA
//!   runs with a shape-anchoring restoring force. At [`START_HEIGHT_FAST`] it
//!   switches to a fast EMA that tracks mature-market volatility.
//! - Output filter (`filter`): below height 630,000 batch-payout transactions
//!   are capped strictly (100 outputs). From there the cap relaxes (250) but
//!   still drops very large fan-outs.
//!
//! The two boundaries differ on purpose. The EMA must hand off to fast before the
//! 2020 crash, while the output cap helps the thin pre-2020 mix for longer and
//! still prevents modern fan-out clusters from dominating one EMA slot.

use std::convert::Infallible;

use brk_types::{Dollars, Histogram};

mod config;
mod filter;
mod scale;
mod seed;
mod stencil;
mod window;

pub use config::{Config, START_HEIGHT_FAST, START_HEIGHT_SLOW};
pub use filter::PaymentFilter;
use scale::NUM_BINS;
pub use scale::{bin_to_cents, cents_to_bin, sats_to_bin};
pub use seed::{FIRST_PRICE_HEIGHT, pre_oracle_prices_from};

pub use stencil::ROUND_DOLLARS;
use stencil::Stencil;
use window::EmaWindow;

/// One block's raw output counts per log-scale bin.
pub type HistogramRaw = Histogram<u32, NUM_BINS>;
/// The window's smoothed EMA per log-scale bin.
pub type HistogramEma = Histogram<f64, NUM_BINS>;

/// Oracle algorithm version. Bump on any change that alters computed prices
/// so downstream consumers can invalidate cached results.
pub const VERSION: u32 = 6;

/// One block's search: its window's first candidate bin and each candidate's score, in bin order.
#[derive(Clone, Debug)]
pub struct Search {
    pub first_bin: usize,
    pub scores: Vec<f64>,
}

#[derive(Clone)]
pub struct Oracle {
    window: EmaWindow,
    ref_bin: f64,
    config: Config,
    warmup: bool,
    stencil: Stencil,
}

impl Oracle {
    fn new(start_bin: f64, config: Config) -> Self {
        Self {
            window: EmaWindow::new(config.window_size, config.alpha),
            ref_bin: start_bin,
            warmup: false,
            stencil: Stencil::new(config.shape_weight),
            config,
        }
    }

    /// Restore an oracle from an exact stored reference bin. `fill` replays the
    /// warmup blocks through `process_histogram`; its error aborts the restore.
    /// During warmup the ring fills without recomputing the EMA or searching;
    /// the EMA is computed once at the end, so the first call after has it
    /// primed. The slow regime's adaptive shape profile starts afresh.
    pub fn from_checkpoint<E>(
        ref_bin: f64,
        config: Config,
        fill: impl FnOnce(&mut Self) -> Result<(), E>,
    ) -> Result<Self, E> {
        let mut oracle = Self::new(ref_bin, config);
        oracle.warmup = true;
        fill(&mut oracle)?;
        oracle.warmup = false;
        oracle.window.recompute();
        Ok(oracle)
    }

    pub fn process_histogram(&mut self, hist: &HistogramRaw) -> f64 {
        self.window.push(hist);

        if !self.warmup {
            self.window.recompute();

            self.ref_bin = self.stencil.pick(
                self.window.ema(),
                self.ref_bin,
                self.config.search_below,
                self.config.search_above,
            );
        }
        self.ref_bin
    }

    /// Switch EMA regime mid-stream (slow -> fast at [`START_HEIGHT_FAST`]) by
    /// re-warming under `config` over the most recent `config.window_size` raw
    /// histograms, so a continuous build and an incremental warm-up reach the
    /// same state. `ref_bin` carries over.
    pub fn reconfigure(&mut self, config: Config) {
        let kept = self.window.recent(config.window_size);
        let Ok(oracle) = Self::from_checkpoint(self.ref_bin, config, |o| {
            kept.iter().for_each(|h| {
                o.process_histogram(h);
            });
            Ok::<_, Infallible>(())
        });
        *self = oracle;
    }

    /// The current weighted EMA over the window, one value per log-scale bin.
    /// `ema()[i]` is bin `i` (see `sats_to_bin`).
    pub fn ema(&self) -> &HistogramEma {
        self.window.ema()
    }

    /// The search behind the last block's pick, `prev_bin` the reference before it: the window's
    /// first candidate bin and each candidate's score, as the oracle ranked them. Exact from
    /// [`START_HEIGHT_FAST`] on; below it, the slow regime's shape profile has already learned
    /// from the pick it scored.
    pub fn search(&self, prev_bin: f64) -> Option<Search> {
        self.stencil
            .scores(
                self.window.ema(),
                prev_bin,
                self.config.search_below,
                self.config.search_above,
            )
            .map(|(first_bin, scores)| Search { first_bin, scores })
    }

    /// The current reference bin: the last block's pick.
    pub fn ref_bin(&self) -> f64 {
        self.ref_bin
    }

    pub fn price_dollars(&self) -> Dollars {
        bin_to_cents(self.ref_bin).into()
    }
}
