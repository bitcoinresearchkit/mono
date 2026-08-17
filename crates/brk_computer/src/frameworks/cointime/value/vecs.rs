use brk_traversable::Traversable;
use brk_types::StoredF64;
use vecdb::{Rw, StorageMode};

use crate::internal::PerBlockCumulativeRolling;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Spot price in USD multiplied by coinblocks destroyed by the block.
    pub destroyed: PerBlockCumulativeRolling<StoredF64, M>,
    /// Spot price in USD multiplied by coinblocks created by the block.
    pub created: PerBlockCumulativeRolling<StoredF64, M>,
    /// Spot price in USD multiplied by net coinblocks stored by the block.
    pub stored: PerBlockCumulativeRolling<StoredF64, M>,
    /// Supply-adjusted value of coin days destroyed: spot price in USD
    /// multiplied by the block's coin days destroyed and divided by circulating
    /// supply in BTC. Returns zero when circulating supply is zero.
    pub vocdd: PerBlockCumulativeRolling<StoredF64, M>,
}
