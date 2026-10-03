use bitview_cohort::AgeRange;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazySpotValuePerBlock, PerBlock};
use brk_types::{BoundedRatio, Height, StoredF64};
use vecdb::{AnyStoredVec, ReadableVec, Rw, StorageMode, WritableVec};

use crate::{Mobility, SpendingExposureSeries};

#[derive(Traversable)]
pub struct AgeRangeVecs<M: StorageMode = Rw> {
    /// Empirical daily spending hazard for each UTXO age range: cumulative
    /// transfer volume in BTC divided by cumulative coin days created in that
    /// range. It estimates the fraction of the range's supply spent per day;
    /// higher values indicate faster turnover. Returns zero when cumulative
    /// coin days created is zero.
    pub spending_rate: AgeRange<PerBlock<StoredF64, M>>,
    /// Estimated remaining-lifetime spending exposure for each UTXO age range.
    /// It integrates observed positive spending hazards from the range midpoint
    /// through subsequent complete ranges, then integrates an exponential tail
    /// fitted by duration-weighted regression of log hazard on age. Returns
    /// zero when a decreasing finite tail cannot be fitted. Larger exposure
    /// implies a greater eventual probability of spending.
    pub spending_exposure: SpendingExposureSeries<M>,
    /// Canonical bounded spending probability, batched by age range.
    #[traversable(hidden)]
    pub mobility_source: AgeRange<CachedSeries<Height, BoundedRatio, M>>,
    pub supply: Mobility<AgeRange<LazySpotValuePerBlock>>,
}

impl AgeRangeVecs {
    pub(crate) fn push(
        &mut self,
        spending_rate: &AgeRange<StoredF64>,
        spending_exposure: &AgeRange<StoredF64>,
        mobility: &AgeRange<BoundedRatio>,
    ) {
        for (target, value) in self.spending_rate.iter_mut().zip(spending_rate.iter()) {
            target.height.push(*value);
        }
        for (target, value) in self
            .spending_exposure
            .age_range
            .iter_mut()
            .zip(spending_exposure.iter())
        {
            target.height.push(*value);
        }
        for (target, value) in self.mobility_source.iter_mut().zip(mobility.iter()) {
            target.push(*value);
        }
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.spending_rate
            .iter_mut()
            .map(|v| &mut v.height as &mut dyn AnyStoredVec)
            .chain(
                self.spending_exposure
                    .age_range
                    .iter_mut()
                    .map(|v| &mut v.height as &mut dyn AnyStoredVec),
            )
            .chain(
                self.mobility_source
                    .iter_mut()
                    .map(|v| v as &mut dyn AnyStoredVec),
            )
    }
}

impl<M: StorageMode> AgeRangeVecs<M> {
    /// Lifetime mobility by height for each age range: the coinflow URPD weight source.
    pub fn urpd_weight_sources(&self) -> AgeRange<&impl ReadableVec<Height, StoredF64>> {
        AgeRange::from_fn(|id| &id.select(&self.spending_exposure.mobility).height)
    }
}
