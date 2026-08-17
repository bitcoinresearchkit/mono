use brk_cohort::{AgeRange, AgeRangeId};
use brk_traversable::Traversable;
use brk_types::{Sats, StoredF64};
use vecdb::{Rw, StorageMode};

use crate::internal::{ColumnarPerBlock, LazyColumnPerBlock, LazyColumnSpotValuePerBlock};

use super::{Mobility, SpendingExposureSeries};

#[derive(Traversable)]
pub struct AgeRangeVecs<M: StorageMode = Rw> {
    /// Empirical daily spending hazard for each UTXO age range: cumulative
    /// transfer volume in BTC divided by cumulative coin days created in that
    /// range. Returns zero when cumulative coin days created is zero.
    pub spending_rate: ColumnarPerBlock<
        StoredF64,
        AgeRangeId,
        AgeRange<LazyColumnPerBlock<StoredF64, AgeRangeId>>,
        M,
    >,
    /// Estimated remaining-lifetime spending exposure for each UTXO age range.
    /// It integrates observed positive spending hazards from the range midpoint
    /// through subsequent complete ranges, then integrates an exponential tail
    /// fitted by duration-weighted regression of log hazard on age. Returns
    /// zero when a decreasing finite tail cannot be fitted.
    pub spending_exposure: ColumnarPerBlock<StoredF64, AgeRangeId, SpendingExposureSeries, M>,
    pub supply: Mobility<
        ColumnarPerBlock<Sats, AgeRangeId, AgeRange<LazyColumnSpotValuePerBlock<AgeRangeId>>, M>,
    >,
}
