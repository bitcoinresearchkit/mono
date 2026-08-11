use brk_traversable::Traversable;
use brk_types::{Cents, StoredF64};
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

use super::{LossPercentileId, Percentiles, PriceBandId, PriceBands, price::LazyColumnPrice};
use crate::internal::{ColumnarDailyMetric, LazyColumnDailyMetric};

#[derive(Deref, DerefMut, Traversable)]
pub struct ModeVecs<M: StorageMode = Rw> {
    pub loss_threshold: ColumnarDailyMetric<
        StoredF64,
        LossPercentileId,
        Percentiles<LazyColumnDailyMetric<StoredF64, LossPercentileId>>,
        M,
    >,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub prices:
        ColumnarDailyMetric<Cents, PriceBandId, PriceBands<LazyColumnPrice<PriceBandId>>, M>,
}
