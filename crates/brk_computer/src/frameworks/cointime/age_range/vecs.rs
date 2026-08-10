use brk_cohort::{AgeRange, AgeRangeId};
use brk_traversable::Traversable;
use brk_types::{Sats, StoredF64};
use vecdb::{Rw, StorageMode};

use crate::internal::{
    ColumnarPerBlock, ColumnarPerBlockCumulativeRolling, LazyColumnPerBlock,
    LazyColumnPerBlockCumulativeRolling, LazyColumnSpotValuePerBlock, LazyPerBlock,
};

#[derive(Clone, Traversable)]
pub struct ActivitySeries {
    pub wakefulness: AgeRange<LazyColumnPerBlock<StoredF64, AgeRangeId>>,
    pub dormancy: AgeRange<LazyPerBlock<StoredF64>>,
    pub wakefulness_to_dormancy: AgeRange<LazyPerBlock<StoredF64>>,
}

#[derive(Traversable)]
pub struct SupplyVecs<T> {
    pub awake: T,
    pub dormant: T,
}

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub coindays_created: ColumnarPerBlockCumulativeRolling<
        StoredF64,
        AgeRangeId,
        AgeRange<LazyColumnPerBlockCumulativeRolling<StoredF64, AgeRangeId>>,
        M,
    >,
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
