use brk_cohort::{AgeRange, AgeRangeId};
use brk_traversable::Traversable;
use brk_types::{Sats, StoredF64};
use vecdb::{Rw, StorageMode};

use crate::internal::{ColumnarPerBlock, LazyColumnPerBlock, LazyColumnSpotValuePerBlock};

use super::{Mobility, SpendingExposureSeries};

#[derive(Traversable)]
pub struct AgeRangeVecs<M: StorageMode = Rw> {
    pub spending_rate: ColumnarPerBlock<
        StoredF64,
        AgeRangeId,
        AgeRange<LazyColumnPerBlock<StoredF64, AgeRangeId>>,
        M,
    >,
    pub spending_exposure: ColumnarPerBlock<StoredF64, AgeRangeId, SpendingExposureSeries, M>,
    pub supply: Mobility<
        ColumnarPerBlock<Sats, AgeRangeId, AgeRange<LazyColumnSpotValuePerBlock<AgeRangeId>>, M>,
    >,
}
