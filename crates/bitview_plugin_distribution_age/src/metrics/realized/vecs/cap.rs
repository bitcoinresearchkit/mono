use bitview_cohort::{CohortContext, CreationCohorts};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyFiatPerBlock;
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CreationSources;

#[derive(Traversable)]
pub struct RealizedCapByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: CreationCohorts<LazyFiatPerBlock<Cents>>,
    #[traversable(hidden)]
    pub stored: CreationSources<Cents, M>,
}

impl RealizedCapByCohort {
    pub fn forced_import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let stored = CreationSources::forced_import(db, "realized_cap_cents", version)?;
        let cohorts = CreationCohorts::new(|cohort_id| {
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
