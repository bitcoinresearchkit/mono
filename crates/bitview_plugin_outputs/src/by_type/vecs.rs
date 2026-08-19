use bitview_traversable::Traversable;
use brk_cohort::{ByType, OutputTypeId};
use brk_types::{PartsPerMillion32, StoredU16, StoredU64};
use vecdb::{Rw, StorageMode};

use super::{CachedSpendableOutputCount, WithOutputTypes};
use bitview_compute::{
    ColumnarPerBlock, ColumnarPerBlockCumulativeRolling, LazyColumnCountPerBlockCumulativeRolling,
    LazyColumnPerBlockCumulativeRolling, LazyPercentCumulativeRolling,
};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Counts of transaction outputs, including coinbase. Per-type series
    /// classify outputs by BRK locking-script type. Column order is P2PK65,
    /// P2PK33, P2PKH, P2MS, P2SH, P2WPKH, P2WSH, P2TR, P2A, unknown, empty,
    /// and `OP_RETURN`.
    pub output_count: ColumnarPerBlock<
        StoredU16,
        OutputTypeId,
        WithOutputTypes<LazyColumnCountPerBlockCumulativeRolling>,
        M,
    >,
    /// Number of transaction outputs excluding `OP_RETURN` outputs, which are
    /// provably unspendable.
    pub spendable_output_count: CachedSpendableOutputCount,
    /// Outputs of the selected BRK locking-script type divided by all outputs
    /// over the same cumulative or trailing window, including coinbase outputs.
    pub output_share: ByType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
    /// Number of transactions containing at least one output of the selected
    /// BRK locking-script type. Each transaction is counted once per type; the
    /// `all` aggregate counts every transaction, including coinbase.
    pub tx_count: ColumnarPerBlockCumulativeRolling<
        StoredU64,
        OutputTypeId,
        WithOutputTypes<LazyColumnPerBlockCumulativeRolling<StoredU64, OutputTypeId>>,
        M,
    >,
    /// Transactions containing the selected output type divided by all
    /// transactions over the same cumulative or trailing window, including
    /// coinbase transactions.
    pub tx_share: ByType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
}
