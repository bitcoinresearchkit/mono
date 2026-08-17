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
    pub rsi: PercentPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    pub(super) rsi_min: PercentPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    pub(super) rsi_max: PercentPerBlock<PartsPerMillion32, M>,
    #[traversable(hidden)]
    pub(super) stoch_rsi: PercentPerBlock<PartsPerMillion32, M>,
    pub stoch_rsi_k: PercentPerBlock<PartsPerMillion32, M>,
    pub stoch_rsi_d: PercentPerBlock<PartsPerMillion32, M>,
}
