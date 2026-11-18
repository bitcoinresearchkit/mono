use bitview_primitives::{CoinBlocks, PartsPerMillion64};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPerBlockCumulativeRolling, LazyRatioRollingWindows, LazyRollingSumsFromHeight,
    PerBlockCumulative, PerBlockCumulativeRolling,
};
use brk_types::{Height, Sats};
use derive_more::{Deref, DerefMut};
use vecdb::{LazyVec, Rw, StorageMode};

use super::DerivedVecs;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Coinblocks created by each block, equal to the circulating supply in BTC
    /// at that height.
    #[traversable(lead = "Coinblocks created: holding time added, one per bitcoin per block.")]
    pub coinblocks_created: CoinblocksCreated<M>,
    /// Coinblocks destroyed by each block: each spent output's BTC value
    /// multiplied by the blocks it stayed unspent.
    #[traversable(lead = "Coinblocks destroyed: holding time spent as coins move.")]
    pub coinblocks_destroyed: LazyPerBlockCumulativeRolling<CoinBlocks>,
    /// Net coinblocks stored. Its cumulative value is cumulative coinblocks
    /// created minus cumulative coinblocks destroyed; its per-block value is
    /// the change in that cumulative stock, negative (as are window sums) when more was
    /// destroyed than created.
    #[traversable(lead = "Coinblocks stored: holding time kept, added minus spent.")]
    pub coinblocks_stored: PerBlockCumulativeRolling<CoinBlocks, M>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub derived: DerivedVecs<M>,
    /// Concurrent liveliness: coinblocks destroyed divided by coinblocks
    /// created within a trailing window. Above one, spending consumed more
    /// holding time than the window added.
    #[traversable(lead = "Concurrent liveliness: holding time spent against holding time added, over a window.")]
    pub concurrent_liveliness: LazyRatioRollingWindows<PartsPerMillion64>,
}

/// Coinblocks created: each block's value is the circulating supply itself, read exactly, not
/// a difference of the f64 running total, which drifts from it by up to half that total's
/// spacing. The running total and the window sums stay stored, so they differ from a sum of
/// block values by that drift (under 2 coinblocks in 1.4e13 at height 976,337).
#[derive(Deref, DerefMut, Traversable)]
#[traversable(client = "Flow")]
pub struct CoinblocksCreated<M: StorageMode = Rw> {
    /// Value for the represented block.
    #[traversable(accumulation = Flow)]
    pub block: LazyVec<Height, CoinBlocks, Sats>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub(crate) running: PerBlockCumulative<CoinBlocks, M>,
    pub sum: LazyRollingSumsFromHeight<CoinBlocks>,
}
