//! Pure on-chain BTC/USD price oracle.
//!
//! Detects round-dollar transaction patterns ($1, $5, $10, ... $10,000) in Bitcoin
//! block outputs to derive the current price without any exchange data.
//!
//! Behavior changes by height along two independent axes, each in its own module:
//!
//! - EMA regime (`config`): below [`START_HEIGHT_SLOW`] prices come from the baked
//!   pre-oracle tape. From there to [`START_HEIGHT_FAST`] a slow cold-start EMA
//!   runs with a shape-anchoring restoring force. At [`START_HEIGHT_FAST`] it
//!   switches to a fast EMA that tracks mature-market volatility.
//! - Output filter (`filter`): below
//!   [`PaymentFilter::MODERN_TX_OUTPUT_FANOUT_CAP_START_HEIGHT`] batch-payout
//!   transactions are capped strictly. Above it the cap relaxes but still drops
//!   very large fan-outs.
//!
//! The two boundaries differ on purpose. The EMA must hand off to fast before the
//! 2020 crash, while the output cap helps the thin pre-2020 mix for longer and
//! still prevents modern fan-out clusters from dominating one EMA slot.

use brk_types::{Cents, Dollars};

mod config;
mod filter;
mod histogram_ema;
mod histogram_ema_compact;
mod histogram_raw;
mod scale;
mod seed;
mod stencil;
mod window;

pub use config::{Config, START_HEIGHT_FAST, START_HEIGHT_SLOW};
pub use filter::PaymentFilter;
pub use histogram_ema::HistogramEma;
pub use histogram_ema_compact::HistogramEmaCompact;
pub use histogram_raw::HistogramRaw;
pub use scale::{BINS_PER_DECADE, NUM_BINS, bin_to_cents, cents_to_bin, sats_to_bin};
pub use seed::{pre_oracle_price_cents, pre_oracle_prices_from, seed_bin, seed_price_cents};

use stencil::Stencil;
use window::EmaWindow;

/// Oracle algorithm version. Bump on any change that alters computed prices
/// so downstream consumers can invalidate cached results.
pub const VERSION: u32 = 4;

#[derive(Clone)]
pub struct Oracle {
    window: EmaWindow,
    ref_bin: f64,
    config: Config,
    warmup: bool,
    stencil: Stencil,
}

impl Oracle {
    pub fn new(start_bin: f64, config: Config) -> Self {
        Self {
            window: EmaWindow::new(config.window_size, config.alpha),
            ref_bin: start_bin,
            warmup: false,
            stencil: Stencil::new(config.shape_weight),
            config,
        }
    }

    /// Create an oracle ready to process height [`START_HEIGHT_SLOW`], seeded from
    /// the baked pre-oracle price tape and using the slow cold-start config.
    pub fn from_seed() -> Self {
        Self::new(seed_bin(), Config::slow())
    }

    /// Create an oracle restored from a known price. `fill` should call
    /// `process_histogram` for the warmup blocks. During warmup the ring
    /// fills without recomputing EMA or searching, then we recompute once
    /// at the end so the first non-warmup call has a primed EMA.
    pub fn from_checkpoint(ref_bin: f64, config: Config, fill: impl FnOnce(&mut Self)) -> Self {
        let mut oracle = Self::new(ref_bin, config);
        oracle.warmup = true;
        fill(&mut oracle);
        oracle.warmup = false;
        oracle.window.recompute();
        oracle
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
        *self = Self::from_checkpoint(self.ref_bin, config, |o| {
            kept.iter().for_each(|h| {
                o.process_histogram(h);
            });
        });
    }

    pub fn ref_bin(&self) -> f64 {
        self.ref_bin
    }

    /// The current weighted EMA over the window, one value per log-scale bin.
    /// `ema()[i]` is bin `i` (see `sats_to_bin`).
    pub fn ema(&self) -> &HistogramEma {
        self.window.ema()
    }

    pub fn price_cents(&self) -> Cents {
        bin_to_cents(self.ref_bin).into()
    }

    pub fn price_dollars(&self) -> Dollars {
        self.price_cents().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_basic() {
        let oracle = Oracle::new(1600.0, Config::default());
        assert_eq!(oracle.ref_bin(), 1600.0);
        assert_eq!(oracle.price_cents(), bin_to_cents(1600.0).into());
    }

    #[test]
    fn from_seed_matches_manual_seed() {
        let mut seeded = Oracle::from_seed();
        let mut manual = Oracle::new(seed_bin(), Config::slow());
        let mut hist = HistogramRaw::zeros();
        hist.increment(1200);

        assert_eq!(seeded.ref_bin(), manual.ref_bin());
        assert_eq!(
            seeded.process_histogram(&hist),
            manual.process_histogram(&hist)
        );
        assert!(seeded.ema().iter().eq(manual.ema().iter()));
    }

    // reconfigure must leave the oracle in the same state as a fresh warm-up
    // over the most recent window of raw histograms. The continuous build and
    // the incremental resume rely on this agreeing at the slow -> fast seam.
    #[test]
    fn reconfigure_matches_fresh_warmup() {
        let hists: Vec<HistogramRaw> = (0..60)
            .map(|i| {
                let mut h = HistogramRaw::zeros();
                h.increment(1200 + i % 7);
                h.increment(1600 + i % 5);
                h
            })
            .collect();

        let fast = Config::default();
        let mut switched = Oracle::new(1600.0, Config::slow());
        hists.iter().for_each(|h| {
            switched.process_histogram(h);
        });
        switched.reconfigure(fast);

        let keep = fast.window_size;
        let fresh = Oracle::from_checkpoint(switched.ref_bin(), fast, |o| {
            hists[hists.len() - keep..].iter().for_each(|h| {
                o.process_histogram(h);
            });
        });

        assert!(switched.ema().iter().eq(fresh.ema().iter()));
    }

    #[test]
    fn sequential_ema_matches_fresh_warmups() {
        let hists: Vec<HistogramRaw> = (0..80)
            .map(|i| {
                let mut h = HistogramRaw::zeros();
                h.increment(1000 + i % 11);
                h.increment(1300 + i % 13);
                h.increment(1700 + i % 17);
                h
            })
            .collect();

        for config in [Config::slow(), Config::default()] {
            let query_start = config.window_size + 5;
            let query_end = query_start + 20;
            let seed = 1600.0;
            let mut sequential = Oracle::from_checkpoint(seed, config, |o| {
                hists[query_start + 1 - config.window_size..query_start + 1]
                    .iter()
                    .for_each(|h| {
                        o.process_histogram(h);
                    });
            });

            for height in query_start..query_end {
                if height != query_start {
                    sequential.process_histogram(&hists[height]);
                }

                let fresh = Oracle::from_checkpoint(seed, config, |o| {
                    hists[height + 1 - config.window_size..height + 1]
                        .iter()
                        .for_each(|h| {
                            o.process_histogram(h);
                        });
                });

                assert!(sequential.ema().iter().eq(fresh.ema().iter()));
            }
        }
    }
}
