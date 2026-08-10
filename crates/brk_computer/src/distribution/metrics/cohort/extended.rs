use brk_cohort::Filter;
use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::Version;
use vecdb::AnyStoredVec;
use vecdb::{Exit, Rw, StorageMode};

use crate::{
    distribution::AllChainCache,
    distribution::metrics::{
        ActivityFull, CohortMetricsBase, CostBasis, ImportConfig, OutputsBase, RealizedFull,
        RelativeWithExtended, SupplyCore, UnrealizedFull,
    },
    price,
};

/// Cohort metrics with extended realized + extended cost basis (no adjusted).
/// Used by: lth, age_range cohorts.
#[derive(Traversable)]
pub struct ExtendedCohortMetrics<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub filter: Filter,
    pub supply: Box<SupplyCore<M>>,
    pub outputs: Box<OutputsBase<M>>,
    pub activity: Box<ActivityFull<M>>,
    pub realized: Box<RealizedFull<M>>,
    pub cost_basis: Box<CostBasis<M>>,
    pub unrealized: Box<UnrealizedFull<M>>,
    #[traversable(flatten)]
    pub relative: Box<RelativeWithExtended<M>>,
}

impl CohortMetricsBase for ExtendedCohortMetrics {
    type ActivityVecs = ActivityFull;
    type RealizedVecs = RealizedFull;
    type UnrealizedVecs = UnrealizedFull;

    impl_cohort_accessors!();

    fn validate_computed_versions(&mut self, base_version: Version) -> Result<()> {
        self.supply.validate_computed_versions(base_version)?;
        self.activity.validate_computed_versions(base_version)?;
        self.cost_basis.validate_computed_versions(base_version)?;
        Ok(())
    }

    fn min_stateful_len(&self) -> usize {
        // Only check per-block pushed vecs, not aggregated ones (supply, outputs,
        // activity, realized core, unrealized core are summed from age_range).
        self.realized
            .min_stateful_len()
            .min(self.unrealized.min_stateful_len())
            .min(self.cost_basis.min_stateful_len())
    }

    fn collect_all_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = Vec::new();
        vecs.extend(self.supply.collect_vecs_mut());
        vecs.extend(self.outputs.collect_vecs_mut());
        vecs.extend(self.activity.collect_vecs_mut());
        vecs.extend(self.realized.collect_vecs_mut());
        vecs.extend(self.cost_basis.collect_vecs_mut());
        vecs.extend(self.unrealized.collect_vecs_mut());
        vecs
    }
}

impl ExtendedCohortMetrics {
    pub(crate) fn forced_import(
        cfg: &ImportConfig,
        supply: SupplyCore,
        all_chain: &AllChainCache,
    ) -> Result<Self> {
        let realized = RealizedFull::forced_import(cfg, all_chain)?;
        let unrealized = UnrealizedFull::forced_import(cfg, &realized.price.ppm)?;

        let relative = RelativeWithExtended::forced_import(cfg, &unrealized, all_chain)?;

        Ok(Self {
            filter: cfg.filter.clone(),
            supply: Box::new(supply),
            outputs: Box::new(OutputsBase::forced_import(cfg)?),
            activity: Box::new(ActivityFull::forced_import(cfg)?),
            realized: Box::new(realized),
            cost_basis: Box::new(CostBasis::forced_import(cfg)?),
            unrealized: Box::new(unrealized),
            relative: Box::new(relative),
        })
    }

    #[allow(clippy::too_many_arguments)]
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
            &self.activity.transfer_volume,
            exit,
        )?;

        self.cost_basis.compute_prices(
            starting_lengths,
            &prices.spot.cents.height,
            &self.unrealized.invested_capital.in_profit.cents.height,
            &self.unrealized.invested_capital.in_loss.cents.height,
            &self.supply.in_profit.sats.height,
            &self.supply.in_loss.sats.height,
            &self.unrealized.capitalized_cap_in_profit_raw,
            &self.unrealized.capitalized_cap_in_loss_raw,
            exit,
        )?;

        self.unrealized
            .compute_sentiment(starting_lengths, &prices.spot.cents.height, exit)?;

        self.relative.compute(
            starting_lengths.height,
            &self.supply,
            &self.unrealized,
            &self.realized,
            &self.supply.total.usd.height,
            exit,
        )?;

        Ok(())
    }
}
