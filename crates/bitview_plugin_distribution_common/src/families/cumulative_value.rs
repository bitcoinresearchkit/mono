use bitview_cohort::{CohortContext, CohortGroup};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CumulativeCohortValueSources, LazyValuePerBlockCumulativeRolling, LazyWindowStartVec, SatsCents,
};
use brk_error::Result;
use brk_types::{Cents, Sats, StoredU64, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct CumulativeValueByCohort<G: CohortGroup, M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// UTXO groups and spent output value grouped by the spending address's
    /// balance immediately before the spend.
    pub cohorts: G::Of<LazyValuePerBlockCumulativeRolling>,
    #[traversable(hidden)]
    pub stored: CumulativeCohortValueSources<G, M>,
}

impl<G: CohortGroup> CumulativeValueByCohort<G>
where
    G::Of<StoredU64>: std::ops::AddAssign + Clone + Default,
{
    pub fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = CumulativeCohortValueSources::forced_import(
            db,
            &format!("{metric}_cumulative"),
            version,
        )?;
        let cohorts = G::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, metric);
            let SatsCents { sats, cents } = stored
                .sources(cohort_id, &name, version)
                .expect("supported stored value cohort");
            LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
                &name,
                version,
                &sats,
                &cents,
                mappings,
                window_starts,
            )
        });
        Ok(Self { cohorts, stored })
    }

    #[inline(always)]
    pub fn push_block(&mut self, sats: &G::Of<Sats>, cents: &G::Of<Cents>) {
        self.stored.push_block(sats, cents);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored.stored_vecs_mut()
    }
}
