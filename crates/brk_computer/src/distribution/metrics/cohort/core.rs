use brk_cohort::Filter;
use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::Version;
use vecdb::{AnyStoredVec, Exit, Rw, StorageMode};

use crate::{
    distribution::metrics::{
        ActivityCore, AllSupplyCache, CohortMetricsBase, ImportConfig, OutputsBase, RealizedCore,
        SupplyCore, UnrealizedCore,
    },
    price,
};

#[derive(Traversable)]
pub struct CoreCohortMetrics<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub filter: Filter,
    pub supply: Box<SupplyCore<M>>,
    pub outputs: Box<OutputsBase<M>>,
    pub activity: Box<ActivityCore<M>>,
    pub realized: Box<RealizedCore<M>>,
    pub unrealized: Box<UnrealizedCore<M>>,
}

impl CoreCohortMetrics {
    pub(crate) fn forced_import(cfg: &ImportConfig, all_supply: &AllSupplyCache) -> Result<Self> {
        let supply = SupplyCore::forced_import(cfg, all_supply)?;
        Self::forced_import_with_supply(cfg, supply)
    }

    pub(crate) fn forced_import_with_supply(
        cfg: &ImportConfig,
        supply: SupplyCore,
    ) -> Result<Self> {
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

    pub(crate) fn min_stateful_len(&self) -> usize {
        self.supply
            .min_len()
            .min(self.outputs.min_len())
            .min(self.activity.min_len())
            .min(self.realized.min_stateful_len())
            .min(self.unrealized.min_stateful_len())
    }

    pub(crate) fn validate_computed_versions(&mut self, base_version: Version) -> Result<()> {
        self.supply.validate_computed_versions(base_version)?;
        self.activity.validate_computed_versions(base_version)?;
        Ok(())
    }

    pub(crate) fn collect_all_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = Vec::new();
        vecs.extend(self.supply.collect_vecs_mut());
        vecs.extend(self.outputs.collect_vecs_mut());
        vecs.extend(self.activity.collect_vecs_mut());
        vecs.extend(self.realized.collect_vecs_mut());
        vecs.extend(self.unrealized.collect_vecs_mut());
        vecs
    }

    /// Aggregate Core-tier fields from CohortMetricsBase sources (e.g. age_range -> under_age/over_age).
    pub(crate) fn compute_from_base_sources<T: CohortMetricsBase>(
        &mut self,
        starting_lengths: &Lengths,
        others: &[&T],
        exit: &Exit,
    ) -> Result<()> {
        self.supply.compute_from_stateful(
            starting_lengths,
            &others.iter().map(|v| v.supply()).collect::<Vec<_>>(),
            exit,
        )?;
        self.outputs.compute_from_stateful(
            starting_lengths,
            &others.iter().map(|v| v.outputs()).collect::<Vec<_>>(),
            exit,
        )?;
        self.activity.compute_from_stateful(
            starting_lengths,
            &others.iter().map(|v| v.activity_core()).collect::<Vec<_>>(),
            exit,
        )?;
        self.realized.compute_from_stateful(
            starting_lengths,
            &others.iter().map(|v| v.realized_core()).collect::<Vec<_>>(),
            exit,
        )?;
        self.unrealized.compute_from_stateful(
            starting_lengths,
            &others
                .iter()
                .map(|v| v.unrealized_core())
                .collect::<Vec<_>>(),
            exit,
        )?;

        Ok(())
    }

    pub(crate) fn compute_rest_part1(
        &mut self,
        prices: &price::Vecs,
        starting_lengths: &Lengths,
        exit: &Exit,
    ) -> Result<()> {
        self.activity
            .compute_rest_part1(prices, starting_lengths, exit)?;

        self.realized.compute_rest_part1(starting_lengths, exit)?;

        self.unrealized.compute_rest(starting_lengths, exit)?;

        Ok(())
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
