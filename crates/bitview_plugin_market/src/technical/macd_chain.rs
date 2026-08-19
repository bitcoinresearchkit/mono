use bitview_traversable::Traversable;
use brk_types::StoredF32;
use vecdb::{Rw, StorageMode};

use bitview_compute::PerBlock;

#[derive(Traversable)]
pub struct MacdChain<M: StorageMode = Rw> {
    /// EMA of spot price in USD per BTC using the chain's fast span. It
    /// recursively applies `alpha = 2 / (span + 1)`, where `span` is the number
    /// of blocks in the corresponding trailing monotonic-time duration.
    pub ema_fast: PerBlock<StoredF32, M>,
    /// EMA of spot price in USD per BTC using the chain's slow span. It
    /// recursively applies `alpha = 2 / (span + 1)`, where `span` is the number
    /// of blocks in the corresponding trailing monotonic-time duration.
    pub ema_slow: PerBlock<StoredF32, M>,
    /// Fast EMA minus slow EMA, in USD per BTC.
    pub line: PerBlock<StoredF32, M>,
    /// EMA of the MACD line using the chain's signal span, in USD per BTC. It
    /// recursively applies `alpha = 2 / (span + 1)`, where `span` is the number
    /// of blocks in the corresponding trailing monotonic-time duration.
    pub signal: PerBlock<StoredF32, M>,
    /// MACD line minus signal line, in USD per BTC.
    pub histogram: PerBlock<StoredF32, M>,
}
