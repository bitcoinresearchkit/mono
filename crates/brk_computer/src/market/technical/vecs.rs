use brk_traversable::Traversable;
use brk_types::PartsPerMillion32;
use vecdb::{Rw, StorageMode};

use super::{MacdChain, RsiChain};
use crate::internal::{RatioPerBlock, WindowsTo1m};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub rsi: WindowsTo1m<RsiChain<M>>,

    pub pi_cycle: RatioPerBlock<PartsPerMillion32, M>,

    /// MACD chains at base intervals of 1, 7, and 30 days. Each chain uses
    /// fast, slow, and signal durations of 12, 26, and 9 times its base
    /// interval, respectively.
    pub macd: WindowsTo1m<MacdChain<M>>,
}
