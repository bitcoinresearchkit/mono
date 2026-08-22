use bitview_traversable::Traversable;
use brk_types::{Cents, StoredF64};
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

use super::{LossPercentileId, Percentiles, PriceBandId, PriceBands, price::LazyColumnPrice};
use bitview_compute::{ColumnarDailyMetric, LazyColumnDailyMetric};

#[derive(Deref, DerefMut, Traversable)]
pub struct ModeVecs<M: StorageMode = Rw> {
    /// Historical supply-in-loss share that Bedrock treats as a stressed
    /// condition for this mode. It is a linearly interpolated percentile of the
    /// mode's prior finite daily loss shares. The represented day is excluded,
    /// and the value is unavailable until its loss share exists and at least
    /// 365 prior observations are available. Stored as a unitless decimal
    /// share.
    pub loss_threshold: ColumnarDailyMetric<
        StoredF64,
        LossPercentileId,
        Percentiles<LazyColumnDailyMetric<StoredF64, LossPercentileId>>,
        M,
    >,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    /// Bedrock maps historically stressed supply-in-loss shares onto the
    /// represented day's mode-weighted distribution of UTXO creation prices to
    /// estimate lower price bands. A UTXO's creation price is Bitcoin's spot
    /// price when that output was created.
    pub prices:
        ColumnarDailyMetric<Cents, PriceBandId, PriceBands<LazyColumnPrice<PriceBandId>>, M>,
}
