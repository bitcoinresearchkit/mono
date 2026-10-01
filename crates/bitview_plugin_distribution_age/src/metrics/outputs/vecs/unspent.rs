use bitview_cohort::{CohortContext, CreationCohorts};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockWithDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};

use crate::metrics::CreationSources;

#[derive(Traversable)]
pub struct UnspentOutputCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts:
        CreationCohorts<LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>>,
    #[traversable(hidden)]
    pub stored: CreationSources<StoredU64, M>,
}

impl UnspentOutputCount {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = CreationSources::forced_import(db, "utxo_count", version)?;
        let cohorts = CreationCohorts::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "utxo_count");
            let source = stored
                .get(cohort_id)
                .expect("unspent-output cohort source")
                .read_only_boxed_clone();
            LazyPerBlockWithDeltas::from_height_source(
                &name,
                version,
                &source,
                Version::TWO,
                mappings,
                window_starts,
            )
        });

        Ok(Self { cohorts, stored })
    }
}
