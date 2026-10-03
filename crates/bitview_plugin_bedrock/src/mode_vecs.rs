use bitview_primitives::{BoundedRatio, StoredF64};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlock, Price};
use brk_types::{Cents, Height};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Rw, StorageMode, WritableVec};

use super::{LossPercentileId, ModeResult, Percentiles, PriceBandId, PriceBands};

#[derive(Deref, DerefMut, Traversable)]
pub struct ModeVecs<M: StorageMode = Rw> {
    /// Historical supply-in-loss share that Bedrock treats as a stressed
    /// condition for this mode. It is a linearly interpolated percentile of the
    /// mode's prior finite block loss shares. The represented block is excluded,
    /// and the value is unavailable until its loss share exists and at least
    /// 52,560 prior block observations are available. Stored as a bounded share and
    /// exposed as a unitless decimal. Calibration remains full precision.
    pub supply_in_loss_threshold: Percentiles<LazyPerBlock<StoredF64, BoundedRatio>>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    /// Bedrock maps historically stressed supply-in-loss shares onto the
    /// represented block's mode-weighted distribution of UTXO creation prices to
    /// estimate lower price bands. A UTXO's creation price is Bitcoin's spot
    /// price when that output was created.
    pub prices: PriceBands<Price<LazyPerBlock<Cents>>>,
    #[traversable(hidden)]
    pub supply_in_loss_threshold_stored: Percentiles<CachedSeries<Height, BoundedRatio, M>>,
    #[traversable(hidden)]
    pub prices_stored: PriceBands<CachedSeries<Height, Cents, M>>,
}

impl ModeVecs {
    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.supply_in_loss_threshold_stored
            .iter_mut()
            .map(|v| v as &mut dyn AnyStoredVec)
            .chain(
                self.prices_stored
                    .iter_mut()
                    .map(|v| v as &mut dyn AnyStoredVec),
            )
    }

    pub(crate) fn push(&mut self, result: &ModeResult) {
        for id in LossPercentileId::ALL {
            id.select_mut(&mut self.supply_in_loss_threshold_stored)
                .push(*id.select(&result.supply_in_loss_threshold));
        }
        for &id in PriceBandId::ALL {
            id.select_mut(&mut self.prices_stored)
                .push(*id.select(&result.prices));
        }
    }
}
