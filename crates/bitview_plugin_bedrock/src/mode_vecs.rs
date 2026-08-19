use bitview_traversable::Traversable;
use brk_types::{Cents, StoredF64};
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

use super::{LossPercentileId, Percentiles, PriceBandId, PriceBands, price::LazyColumnPrice};
use bitview_compute::{ColumnarDailyMetric, LazyColumnDailyMetric};

#[derive(Deref, DerefMut, Traversable)]
pub struct ModeVecs<M: StorageMode = Rw> {
    /// Linearly interpolated 95th, 98th, 99th, 99.5th, and 99.9th percentiles
    /// of the mode's prior finite daily supply-in-loss shares, in that column
    /// order. The current day is excluded from its calibration history and a
    /// value is unavailable until the current loss share exists and at least
    /// 365 prior observations are available. Stored as unitless decimal shares.
    pub loss_threshold: ColumnarDailyMetric<
        StoredF64,
        LossPercentileId,
        Percentiles<LazyColumnDailyMetric<StoredF64, LossPercentileId>>,
        M,
    >,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    /// Daily price bands derived from the mode's URPD. The five floor bands are
    /// the first ascending creation prices where the share of supply remaining
    /// above the price is at or below the corresponding calibrated loss
    /// threshold. The nine level bands are the 10th through 90th percentiles of
    /// supply at or above the 95th-percentile floor. The stored matrix column
    /// order is the five floors followed by the nine levels.
    pub prices:
        ColumnarDailyMetric<Cents, PriceBandId, PriceBands<LazyColumnPrice<PriceBandId>>, M>,
}
