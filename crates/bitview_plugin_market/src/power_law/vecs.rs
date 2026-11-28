use bitview_compute::Retained;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, PerBlock, Price};
use brk_types::Cents;
use vecdb::{Rw, StorageMode};

use super::compute::Floor;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Harold Christopher Burger's power law ("Bitcoin's natural long-term power-law corridor of
    /// growth", 2019): `log10(price) = −17.01593313 + 5.84509376 · log10(days)`, days counted whole
    /// from the genesis block's day (2009-01-03) to the block's day (UTC). His published
    /// constants, so each value depends only on its day. Null on the genesis day.
    #[traversable(lead = "Burger's power law: his fit of the price to the days since genesis.")]
    pub burger: Price<LazyPerBlock<Cents>>,
    /// Harold Christopher Burger's power-law support line ("Bitcoin's natural long-term
    /// power-law corridor of growth", 2019): `log10(price) = −17.01593313 + 5.84509376 ·
    /// log10(days) − 0.45`, days counted whole from the genesis block's day (2009-01-03) to the
    /// block's day (UTC). Burger publishes the trend's two constants and gives the support as the
    /// trend moved lower with the same slope; 0.45 is read off that support chart (measured
    /// 0.448–0.453). Each value depends only on its day. Null on the genesis day.
    #[traversable(lead = "Burger's power-law floor: his support line, 0.45 in log10 under his power law.")]
    pub burger_floor: Price<LazyPerBlock<Cents>>,
    /// Alexander Mooij's power-law floor (btcpowerlaw.nl): `0.42 × 10^−16.493 ×
    /// days^5.688`, days counted whole from the genesis block's day (2009-01-03) to the block's
    /// day (UTC), as his site's code computes it. His published constants, so each value depends
    /// only on its day. Null on the genesis day.
    #[traversable(lead = "Mooij's power-law floor: 0.42 of his power-law trend.")]
    pub mooij_floor: Price<LazyPerBlock<Cents>>,
    /// The Rainbow chart's lowest band (blockchaincenter.net): `10^(2.775 · ln(n + 1175) −
    /// 19.463)`, `n` the days from 2012-01-01 to the block's day (UTC), the bottom line its chart
    /// draws, as the site's code computes it (read 2026-11-27). The site's text quotes a lower
    /// "bottom" than this plotted line. Its constants are published, so each value depends only
    /// on its day. Null before 2012-01-01, where the chart starts.
    #[traversable(lead = "The Rainbow chart's lowest band: the bottom line it draws.")]
    pub rainbow_lowest_band: Price<LazyPerBlock<Cents>>,
    /// Sminston With's power-law floor (sminstonwith.com), his method run on Bitview's own
    /// price: the 5% quantile regression of log price on log days since the genesis block's day
    /// (2009-01-03), fit exactly on the daily closes (UTC) of the days before the block's day, and
    /// read at the block's day. Each value uses only the prices before it and is never redrawn
    /// with later ones. On 2026-11-27 the fit on every close sat within 1% of his published line.
    /// Null before a year of daily closes (2011-08-16).
    #[traversable(lead = "Sminston With's power-law floor: his 5% quantile line, fit each day on the closes before it.")]
    pub sminston_floor: Price<PerBlock<Cents, M>>,
    pub(super) sminston_state: M::WriteOnly<Retained<Floor>>,
}
