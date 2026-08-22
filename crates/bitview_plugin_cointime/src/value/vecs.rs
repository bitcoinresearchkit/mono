use bitview_traversable::Traversable;
use brk_types::StoredF64;
use vecdb::{Rw, StorageMode};

use bitview_compute::PerBlockCumulativeRolling;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Spot price in USD multiplied by coinblocks destroyed by the block. This
    /// values the accumulated holding time consumed by that block's spending
    /// and is reported in USD-block intervals.
    pub destroyed: PerBlockCumulativeRolling<StoredF64, M>,
    /// Spot price in USD multiplied by coinblocks created by the block. This
    /// values the one-block interval of holding time added by the circulating
    /// supply and is reported in USD-block intervals.
    pub created: PerBlockCumulativeRolling<StoredF64, M>,
    /// Spot price in USD multiplied by net coinblocks stored by the block, where
    /// net stored coinblocks equal coinblocks created minus coinblocks destroyed.
    /// Positive values mean more price-weighted holding time was added than
    /// consumed; negative values mean spending consumed more than was added.
    /// Reported in USD-block intervals.
    pub stored: PerBlockCumulativeRolling<StoredF64, M>,
    /// Supply-adjusted value of coin days destroyed: spot price in USD
    /// multiplied by the block's coin days destroyed and divided by circulating
    /// supply in BTC. Reported in USD-days per BTC of circulating supply. Returns
    /// zero when circulating supply is zero.
    pub vocdd: PerBlockCumulativeRolling<StoredF64, M>,
}
