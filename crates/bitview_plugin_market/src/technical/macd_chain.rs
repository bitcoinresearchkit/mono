use bitview_traversable::Traversable;
use bitview_vecs::PerBlock;
use brk_types::Dollars;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct MacdChain<M: StorageMode = Rw> {
    /// Exponential moving average (EMA) of spot price in USD per BTC using the
    /// chain's fast span. It
    /// recursively applies `alpha = 2 / (span + 1)`, where `span` is the number
    /// of blocks in the corresponding trailing monotonic-time duration.
    pub ema_fast: PerBlock<Dollars, M>,
    /// Exponential moving average (EMA) of spot price in USD per BTC using the
    /// chain's slow span. It
    /// recursively applies `alpha = 2 / (span + 1)`, where `span` is the number
    /// of blocks in the corresponding trailing monotonic-time duration.
    pub ema_slow: PerBlock<Dollars, M>,
    /// Moving average convergence/divergence (MACD) line: fast EMA minus slow
    /// EMA, in USD per BTC. Positive values mean the faster price trend is
    /// above the slower trend; negative values mean it is below.
    pub line: PerBlock<Dollars, M>,
    /// EMA of the MACD line using the chain's signal span, in USD per BTC. It
    /// recursively applies `alpha = 2 / (span + 1)`, where `span` is the number
    /// of blocks in the corresponding trailing monotonic-time duration.
    pub signal: PerBlock<Dollars, M>,
    /// MACD histogram: MACD line minus signal line, in USD per BTC. Positive
    /// values place MACD above its signal; negative values place it below.
    pub histogram: PerBlock<Dollars, M>,
}
