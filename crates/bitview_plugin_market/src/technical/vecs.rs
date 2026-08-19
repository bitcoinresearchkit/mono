use bitview_traversable::Traversable;
use brk_types::PartsPerMillion32;
use vecdb::{Rw, StorageMode};

use super::{MacdChain, RsiChain};
use bitview_compute::{RatioPerBlock, WindowsTo1m};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Relative Strength Index chains at base intervals of 1, 7, and 30 days.
    /// Each RSI uses Wilder-smoothed gains and losses over 14 times its base
    /// interval; its stochastic K and D lines use successive simple averages
    /// over three times that interval.
    pub rsi: WindowsTo1m<RsiChain<M>>,

    /// Pi Cycle ratio: the 111-day simple moving average of Bitcoin spot price
    /// divided by twice its 350-day simple moving average.
    pub pi_cycle: RatioPerBlock<PartsPerMillion32, M>,

    /// MACD chains at base intervals of 1, 7, and 30 days. Each chain uses
    /// fast, slow, and signal durations of 12, 26, and 9 times its base
    /// interval, respectively.
    pub macd: WindowsTo1m<MacdChain<M>>,
}
