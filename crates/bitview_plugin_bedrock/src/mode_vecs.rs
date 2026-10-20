use bitview_primitives::{BoundedRatio, Percent};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, Price};
use brk_types::{Cents, Height};
use vecdb::{AnyStoredVec, Rw, StorageMode, WritableVec};

use super::{FloorVecs, ModeResult, Percentiles, PriceBands};

#[derive(Traversable)]
pub struct ModeVecs<M: StorageMode = Rw> {
    /// Historical supply-in-loss share that Bedrock treats as a stressed
    /// condition for this mode. It is a nearest-rank percentile of the
    /// mode's prior finite block loss shares. The represented block is excluded,
    /// and the value is unavailable until its loss share exists and at least
    /// 52,560 prior block observations are available. Stored as a bounded share and
    /// exposed as a percentage. Calibration remains full precision.
    pub supply_in_loss_share_threshold: Percentiles<LazyPerBlock<Percent, BoundedRatio>>,
    #[traversable(flatten)]
    /// Bedrock maps historically stressed supply-in-loss shares onto the
    /// represented block's mode-weighted distribution of UTXO creation prices to
    /// estimate lower price bands. A UTXO's creation price is Bitcoin's spot
    /// price when that output was created.
    pub prices: PriceBands<Price<LazyPerBlock<Cents>>>,
    #[traversable(flatten)]
    pub floor: FloorVecs<M>,
    #[traversable(hidden)]
    pub(crate) supply_in_loss_share_threshold_stored:
        Percentiles<CachedSeries<Height, BoundedRatio, M>>,
    #[traversable(hidden)]
    pub(crate) prices_stored: PriceBands<CachedSeries<Height, Cents, M>>,
}

impl ModeVecs {
    /// The vecs the replay stores: the thresholds and the prices.
    pub(crate) fn replay_stored_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.supply_in_loss_share_threshold_stored
            .iter_any_stored_mut()
            .chain(self.prices_stored.iter_any_stored_mut())
    }

    pub(crate) fn push(&mut self, result: &ModeResult) {
        for (vec, &value) in self
            .supply_in_loss_share_threshold_stored
            .iter_mut()
            .zip(result.supply_in_loss_threshold.iter())
        {
            vec.push(value);
        }
        for (vec, &value) in self.prices_stored.iter_mut().zip(result.prices.iter()) {
            vec.push(value);
        }
    }
}
