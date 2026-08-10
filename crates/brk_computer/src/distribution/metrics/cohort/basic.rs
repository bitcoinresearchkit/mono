use brk_cohort::Filter;
use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use vecdb::{AnyStoredVec, Exit, Rw, StorageMode};

use crate::{
    distribution::metrics::{
        ActivityCore, CohortMetricsBase, ImportConfig, OutputsBase, RealizedCore, SupplyCore,
        UnrealizedCore,
    },
    price,
};

/// Basic cohort metrics: no extensions, used by age_range cohorts.
#[derive(Traversable)]
pub struct BasicCohortMetrics<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub filter: Filter,
    pub supply: Box<SupplyCore<M>>,
    pub outputs: Box<OutputsBase<M>>,
    pub activity: Box<ActivityCore<M>>,
    pub realized: Box<RealizedCore<M>>,
    pub unrealized: Box<UnrealizedCore<M>>,
}

impl CohortMetricsBase for BasicCohortMetrics {
    type ActivityVecs = ActivityCore;
    type RealizedVecs = RealizedCore;
    type UnrealizedVecs = UnrealizedCore;

    impl_cohort_accessors!();

    fn collect_all_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = Vec::new();
        vecs.extend(self.supply.collect_vecs_mut());
        vecs.extend(self.outputs.collect_vecs_mut());
        vecs.extend(self.activity.collect_vecs_mut());
        vecs.extend(self.realized.collect_vecs_mut());
        vecs.extend(self.unrealized.collect_vecs_mut());
        vecs
    }
}

impl BasicCohortMetrics {
    pub(crate) fn forced_import(cfg: &ImportConfig, supply: SupplyCore) -> Result<Self> {
        let realized = RealizedCore::forced_import(cfg)?;
        let unrealized = UnrealizedCore::forced_import(cfg, &realized.price.ppm)?;

        Ok(Self {
            filter: cfg.filter.clone(),
            supply: Box::new(supply),
            outputs: Box::new(OutputsBase::forced_import(cfg)?),
            activity: Box::new(ActivityCore::forced_import(cfg)?),
            realized: Box::new(realized),
            unrealized: Box::new(unrealized),
        })
    }

    pub(crate) fn compute_rest_part2(
        &mut self,
        prices: &price::Vecs,
        starting_lengths: &Lengths,
        exit: &Exit,
    ) -> Result<()> {
        self.realized.compute_rest_part2(
            prices,
            starting_lengths,
            &self.supply.total.btc.height,
            &self.activity.transfer_volume.sum._24h.cents.height,
            exit,
        )?;

        Ok(())
    }
}
