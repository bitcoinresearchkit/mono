use brk_cohort::{ByType, OutputTypeId};
use brk_traversable::Traversable;
use brk_types::{PartsPerMillion32, StoredU16, StoredU64};
use vecdb::{Rw, StorageMode};

use super::{CachedSpendableOutputCount, WithOutputTypes};
use crate::internal::{
    ColumnarPerBlock, ColumnarPerBlockCumulativeRolling, LazyColumnCountPerBlockCumulativeRolling,
    LazyColumnPerBlockCumulativeRolling, LazyPercentCumulativeRolling,
};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Per-block counts of transaction outputs, including coinbase, partitioned
    /// by BRK locking-script type. Column order is P2PK65, P2PK33, P2PKH, P2MS,
    /// P2SH, P2WPKH, P2WSH, P2TR, P2A, unknown, empty, and `OP_RETURN`.
    pub output_count: ColumnarPerBlock<
        StoredU16,
        OutputTypeId,
        WithOutputTypes<LazyColumnCountPerBlockCumulativeRolling>,
        M,
    >,
    pub spendable_output_count: CachedSpendableOutputCount,
    pub output_share: ByType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
    pub tx_count: ColumnarPerBlockCumulativeRolling<
        StoredU64,
        OutputTypeId,
        WithOutputTypes<LazyColumnPerBlockCumulativeRolling<StoredU64, OutputTypeId>>,
        M,
    >,
    pub tx_share: ByType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
}
