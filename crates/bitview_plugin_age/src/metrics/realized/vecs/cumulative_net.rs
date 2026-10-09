use bitview_cohort::{CohortContext, CreationCohorts};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PartsPerMillionSigned64;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlockCumulativeWithSumsAndDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{CentsSigned, Version};
use vecdb::{Database, Rw, StorageMode};

use crate::metrics::CumulativeCreationSources;

#[derive(Traversable)]
pub struct CumulativeNetRealizedByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: CreationCohorts<
        LazyFiatPerBlockCumulativeWithSumsAndDeltas<
            CentsSigned,
            CentsSigned,
            PartsPerMillionSigned64,
        >,
    >,
    #[traversable(hidden)]
    pub stored: CumulativeCreationSources<CentsSigned, M>,
}

impl CumulativeNetRealizedByCohort {
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let stored =
            CumulativeCreationSources::import(db, "net_realized_pnl_cumulative_cents", version)?;
        let cohorts = CreationCohorts::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "net_realized_pnl");
            let source = stored
                .stored
                .get(cohort_id)
                .expect("supported net realized cohort");
            LazyFiatPerBlockCumulativeWithSumsAndDeltas::from_cumulative_cents_source(
                &name,
                version,
                source,
                Version::new(5),
                mappings,
                window_starts,
            )
        });
        Ok(Self { cohorts, stored })
    }
}
