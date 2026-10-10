use bitview_primitives::{CoinBlocks, PartsPerMillion64};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPerBlockCumulativeRolling, LazyRatioRollingWindows, PerBlockCumulativeRolling,
};
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

use super::DerivedVecs;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Coinblocks created by each block, equal to the circulating supply in BTC
    /// at that height. One coinblock is one BTC held for one block interval.
    pub coinblocks_created: PerBlockCumulativeRolling<CoinBlocks, M>,
    /// Coinblocks destroyed by each block: each spent output's BTC value
    /// multiplied by the blocks it stayed unspent (the age plugin's series).
    pub coinblocks_destroyed: LazyPerBlockCumulativeRolling<CoinBlocks>,
    /// Net coinblocks stored. Its cumulative value is cumulative coinblocks
    /// created minus cumulative coinblocks destroyed; its per-block value is
    /// the change in that cumulative stock.
    pub coinblocks_stored: PerBlockCumulativeRolling<CoinBlocks, M>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub derived: DerivedVecs<M>,
    /// Concurrent liveliness: coinblocks destroyed divided by coinblocks
    /// created within a trailing window. Above one, spending consumed more
    /// holding time than the window added.
    pub concurrent_liveliness: LazyRatioRollingWindows<PartsPerMillion64>,
}
