use bitview_primitives::{CoinBlocks, PartsPerMillion64};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CumulativeState, LazyPerBlockCumulativeRolling, LazyRatioRollingWindows,
    PerBlockCumulativeRolling,
};
use brk_types::{Height, Sats};
use derive_more::{Deref, DerefMut};
use vecdb::{DeltaSub, LazyVec, Rw, StorageMode};

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
    #[traversable(
        lead = "Concurrent liveliness: holding time spent against holding time added, over a window."
    )]
    pub concurrent_liveliness: LazyRatioRollingWindows<PartsPerMillion64>,
    pub(crate) created_sat_blocks: M::WriteOnly<CumulativeState<u128>>,
}

/// The block value reads circulating supply directly. The cumulative value rounds an exact
/// sat-block total to coinblocks, and window sums subtract the rounded endpoints.
pub type CoinblocksCreated<M = Rw> =
    PerBlockCumulativeRolling<CoinBlocks, M, DeltaSub, LazyVec<Height, CoinBlocks, Sats>>;
