use brk_traversable::Traversable;
use brk_types::{PartsPerMillion32, StoredF32};
use vecdb::{Rw, StorageMode};

use crate::internal::{LazyPerBlock, PerBlock, PercentPerBlock};

#[derive(Traversable)]
pub struct RsiChain<M: StorageMode = Rw> {
    #[traversable(hidden)]
    pub(super) gains: LazyPerBlock<StoredF32>,
    #[traversable(hidden)]
    pub(super) losses: LazyPerBlock<StoredF32>,
    #[traversable(hidden)]
    pub(super) average_gain: PerBlock<StoredF32, M>,
    #[traversable(hidden)]
    pub(super) average_loss: PerBlock<StoredF32, M>,
    /// Wilder-smoothed average gain divided by the sum of Wilder-smoothed
    /// average gain and loss. Returns 50% when both averages are zero.
    pub rsi: PercentPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    pub(super) rsi_min: PercentPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    pub(super) rsi_max: PercentPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    pub(super) stoch_rsi: PercentPerBlock<PartsPerMillion32, M>,
    /// Simple moving average of Stochastic RSI over three times the chain's
    /// base interval. Stochastic RSI locates RSI within its trailing RSI range.
    pub stoch_rsi_k: PercentPerBlock<PartsPerMillion32, M>,
    /// Simple moving average of the Stochastic RSI K line over three times the
    /// chain's base interval.
    pub stoch_rsi_d: PercentPerBlock<PartsPerMillion32, M>,
}
