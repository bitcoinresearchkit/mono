use bitview_cohort::{CohortContext, CohortId, UTXOGroupsWithoutAmountOrType};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockWithDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Height, PartsPerMillionSigned64, StoredI64, StoredU64, Version};
use vecdb::{Database, ReadableBoxedVec, ReadableCloneableVec, Rw, StorageMode};

use crate::metrics::UTXOCoreSources;

#[derive(Traversable)]
pub struct UnspentOutputCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UTXOGroupsWithoutAmountOrType<
        LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>,
    >,
    #[traversable(hidden)]
    pub stored: UTXOCoreSources<StoredU64, M>,
}

impl UnspentOutputCount {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        all_count: &ReadableBoxedVec<Height, StoredU64>,
    ) -> Result<Self> {
        let stored = UTXOCoreSources::forced_import_without_all(db, "utxo_count", version)?;
        let cohorts = UTXOGroupsWithoutAmountOrType::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "utxo_count");
            let source = if cohort_id == CohortId::All {
                all_count.read_only_boxed_clone()
            } else {
                stored
                    .get(cohort_id)
                    .expect("unspent-output cohort source")
                    .read_only_boxed_clone()
            };
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
