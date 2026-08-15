use brk_cohort::{AgeRange, AgeRangeId};
use brk_traversable::Traversable;
use brk_types::{Sats, StoredF64};
use vecdb::{Rw, StorageMode};

use crate::internal::{
    ColumnarPerBlock, ColumnarPerBlockCumulativeRolling, LazyColumnPerBlockCumulativeRolling,
    LazyColumnSpotValuePerBlock,
};

use super::{ActivitySeries, SupplyVecs};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub coindays_consumed: ColumnarPerBlockCumulativeRolling<
        StoredF64,
        AgeRangeId,
        AgeRange<LazyColumnPerBlockCumulativeRolling<StoredF64, AgeRangeId>>,
        M,
    >,
    pub coindays_stored: ColumnarPerBlockCumulativeRolling<
        StoredF64,
        AgeRangeId,
        AgeRange<LazyColumnPerBlockCumulativeRolling<StoredF64, AgeRangeId>>,
        M,
    >,
    pub activity: ColumnarPerBlock<StoredF64, AgeRangeId, ActivitySeries, M>,
    pub supply: SupplyVecs<
        ColumnarPerBlock<Sats, AgeRangeId, AgeRange<LazyColumnSpotValuePerBlock<AgeRangeId>>, M>,
    >,
}
