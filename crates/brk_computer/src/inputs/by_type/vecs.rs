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
