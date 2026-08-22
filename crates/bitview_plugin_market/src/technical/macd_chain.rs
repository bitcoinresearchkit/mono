use bitview_traversable::Traversable;
use brk_types::StoredF32;
use vecdb::{Rw, StorageMode};

use bitview_compute::PerBlock;

#[derive(Traversable)]
pub struct MacdChain<M: StorageMode = Rw> {
    /// Exponential moving average (EMA) of spot price in USD per BTC using the
    /// chain's fast span. It
    /// recursively applies `alpha = 2 / (span + 1)`, where `span` is the number
    /// of blocks in the corresponding trailing monotonic-time duration.
    pub ema_fast: PerBlock<StoredF32, M>,
    /// Exponential moving average (EMA) of spot price in USD per BTC using the
    /// chain's slow span. It
    /// recursively applies `alpha = 2 / (span + 1)`, where `span` is the number
    /// of blocks in the corresponding trailing monotonic-time duration.
    pub ema_slow: PerBlock<StoredF32, M>,
    /// Moving average convergence/divergence (MACD) line: fast EMA minus slow
    /// EMA, in USD per BTC. Positive values mean the faster price trend is
    /// above the slower trend; negative values mean it is below.
    pub line: PerBlock<StoredF32, M>,
    /// EMA of the MACD line using the chain's signal span, in USD per BTC. It
    /// recursively applies `alpha = 2 / (span + 1)`, where `span` is the number
    /// of blocks in the corresponding trailing monotonic-time duration.
    pub signal: PerBlock<StoredF32, M>,
    /// MACD histogram: MACD line minus signal line, in USD per BTC. Positive
    /// values place MACD above its signal; negative values place it below.
    pub histogram: PerBlock<StoredF32, M>,
}
