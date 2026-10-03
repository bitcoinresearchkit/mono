use bitview_cohort::{CohortContext, CohortGroup};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CohortSources, LazyFiatPerBlock};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct RealizedCapByCohort<G: CohortGroup, M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: G::Of<LazyFiatPerBlock<Cents>>,
    #[traversable(hidden)]
    pub stored: CohortSources<G, Cents, M>,
}

impl<G: CohortGroup> RealizedCapByCohort<G> {
    pub fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let stored = CohortSources::import(db, "realized_cap_cents", version)?;
        let cohorts = G::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "realized_cap");
            LazyFiatPerBlock::from_cents_source(
                &name,
                version,
                stored.get(cohort_id).expect("realized-cap cohort source"),
                mappings,
            )
        });
        Ok(Self { cohorts, stored })
    }
}
