use brk_cohort::{SpendableType, SpendableTypeId};
use brk_traversable::Traversable;
use brk_types::{PartsPerMillion32, StoredU16, StoredU64};
use vecdb::{Rw, StorageMode};

use super::WithInputTypes;
use crate::internal::{
    ColumnarPerBlock, ColumnarPerBlockCumulativeRolling, LazyColumnCountPerBlockCumulativeRolling,
    LazyColumnPerBlockCumulativeRolling, LazyPercentCumulativeRolling,
};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Per-block counts of non-coinbase transaction inputs, partitioned by the
    /// BRK output type of the previous output they spend. Column order is
    /// P2PK65, P2PK33, P2PKH, P2MS, P2SH, P2WPKH, P2WSH, P2TR, P2A, unknown,
    /// and empty; `OP_RETURN` is excluded because it is unspendable.
    pub input_count: ColumnarPerBlock<
        StoredU16,
        SpendableTypeId,
        WithInputTypes<LazyColumnCountPerBlockCumulativeRolling>,
        M,
    >,
    pub input_share: SpendableType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
    pub tx_count: ColumnarPerBlockCumulativeRolling<
        StoredU64,
        SpendableTypeId,
        WithInputTypes<LazyColumnPerBlockCumulativeRolling<StoredU64, SpendableTypeId>>,
        M,
    >,
    pub tx_share: SpendableType<LazyPercentCumulativeRolling<PartsPerMillion32>>,
}
