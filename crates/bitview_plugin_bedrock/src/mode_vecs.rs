use bitview_traversable::Traversable;
use brk_types::{BoundedRatio, Cents, Day1, StoredF64};
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

use super::{Percentiles, PriceBands};
use bitview_vecs::{CachedSeries, LazyDailyMetric, LazyDailyPrice};

#[derive(Deref, DerefMut, Traversable)]
pub struct ModeVecs<M: StorageMode = Rw> {
    /// Historical supply-in-loss share that Bedrock treats as a stressed
    /// condition for this mode. It is a linearly interpolated percentile of the
    /// mode's prior finite daily loss shares. The represented day is excluded,
    /// and the value is unavailable until its loss share exists and at least
    /// 365 prior observations are available. Stored as a bounded share and
    /// exposed as a unitless decimal. Calibration remains full precision.
    pub supply_in_loss_threshold: Percentiles<LazyDailyMetric<StoredF64, BoundedRatio>>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    /// Bedrock maps historically stressed supply-in-loss shares onto the
    /// represented day's mode-weighted distribution of UTXO creation prices to
    /// estimate lower price bands. A UTXO's creation price is Bitcoin's spot
    /// price when that output was created.
    pub prices: PriceBands<LazyDailyPrice>,
    #[traversable(hidden)]
    pub supply_in_loss_threshold_stored: Percentiles<CachedSeries<Day1, BoundedRatio, M>>,
    #[traversable(hidden)]
    pub prices_stored: PriceBands<CachedSeries<Day1, Cents, M>>,
}
