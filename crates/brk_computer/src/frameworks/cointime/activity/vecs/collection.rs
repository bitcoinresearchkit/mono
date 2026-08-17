use brk_traversable::Traversable;
use brk_types::StoredF64;
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

use crate::internal::PerBlockCumulativeRolling;

use super::DerivedVecs;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Coinblocks created by each block, equal to the circulating supply in BTC
    /// at that height. One coinblock is one BTC held for one block interval.
    pub coinblocks_created: PerBlockCumulativeRolling<StoredF64, M>,
    /// Net coinblocks stored. Its cumulative value is cumulative coinblocks
    /// created minus cumulative coinblocks destroyed; its per-block value is
    /// the change in that cumulative stock.
    pub coinblocks_stored: PerBlockCumulativeRolling<StoredF64, M>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub derived: DerivedVecs<M>,
}
