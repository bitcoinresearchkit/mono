use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Exit, ReadableVec, Rw, StorageMode};

use crate::{
    distribution::AllChainCache,
    distribution::metrics::{
        ActivityFull, AdjustedSopr, CohortMetricsBase, ImportConfig, RealizedFull, SupplyCore,
        UnrealizedFull,
    },
    price,
};

use super::ExtendedCohortMetrics;

/// Cohort metrics with extended + adjusted realized, extended cost basis.
/// Wraps `ExtendedCohortMetrics` and adds adjusted SOPR as a composable add-on.
/// Used by: sth cohort.
#[derive(Deref, DerefMut, Traversable)]
pub struct ExtendedAdjustedCohortMetrics<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub inner: ExtendedCohortMetrics<M>,
    #[traversable(wrap = "realized/sopr", rename = "adjusted")]
    pub asopr: Box<AdjustedSopr<M>>,
}

impl CohortMetricsBase for ExtendedAdjustedCohortMetrics {
    type ActivityVecs = ActivityFull;
    type RealizedVecs = RealizedFull;
    type UnrealizedVecs = UnrealizedFull;

    impl_cohort_accessors_inner!();

    fn validate_computed_versions(&mut self, base_version: Version) -> Result<()> {
        self.inner.validate_computed_versions(base_version)
    }

    fn min_stateful_len(&self) -> usize {
        self.inner.min_stateful_len()
    }

    fn collect_all_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.inner.collect_all_vecs_mut()
    }
}

impl ExtendedAdjustedCohortMetrics {
    pub(crate) fn forced_import(
        cfg: &ImportConfig,
        supply: SupplyCore,
        all_chain: &AllChainCache,
    ) -> Result<Self> {
        let inner = ExtendedCohortMetrics::forced_import(cfg, supply, all_chain)?;
        let asopr = AdjustedSopr::forced_import(cfg)?;
        Ok(Self {
            inner,
            asopr: Box::new(asopr),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute_rest_part2(
        &mut self,
        prices: &price::Vecs,
        starting_lengths: &Lengths,
        under_1h_value_created: &impl ReadableVec<Height, Cents>,
        under_1h_value_destroyed: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.inner
            .compute_rest_part2(prices, starting_lengths, exit)?;

        self.asopr.compute_rest_part2(
            starting_lengths,
            &self
                .inner
                .activity
                .transfer_volume
                .inner
                .cumulative
                .cents
                .height,
            &self
                .inner
                .realized
                .core
                .sopr
                .value_destroyed
                .cumulative
                .height,
            under_1h_value_created,
            under_1h_value_destroyed,
            exit,
        )?;

        Ok(())
    }
}
