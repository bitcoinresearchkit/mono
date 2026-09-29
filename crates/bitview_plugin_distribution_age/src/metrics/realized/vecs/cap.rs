use bitview_cohort::{CohortContext, UTXOGroupsWithoutAmountOrType};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlockWithDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Cents, CentsSigned, PartsPerMillionSigned64, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::UTXOCoreSources;

#[derive(Traversable)]
pub struct RealizedCapByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<
        LazyFiatPerBlockWithDeltas<Cents, CentsSigned, PartsPerMillionSigned64>,
    >,
    #[traversable(hidden)]
    pub stored: UTXOCoreSources<Cents, M>,
}

impl RealizedCapByCohort {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = UTXOCoreSources::forced_import(db, "realized_cap_cents", version)?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "realized_cap");
            LazyFiatPerBlockWithDeltas::from_cents_source(
                &name,
                version,
                stored.get(cohort_id).expect("realized-cap cohort source"),
                Version::TWO,
                mappings,
                window_starts,
            )
        });

        Ok(Self { cohorts, stored })
    }
}
