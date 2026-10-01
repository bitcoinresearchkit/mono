use crate::groups::UtxoGroups;
use crate::sources::UtxoSources;
use bitview_cohort::CohortContext;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockWithDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};
use vecdb::{Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct UnspentOutputCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UtxoGroups<LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>>,
    #[traversable(hidden)]
    pub stored: UtxoSources<StoredU64, M>,
}

impl UnspentOutputCount {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = UtxoSources::forced_import(db, "utxo_count", version)?;
        let cohorts = UtxoGroups::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "utxo_count");
            LazyPerBlockWithDeltas::from_height_source(
                &name,
                version,
                stored.get(cohort_id).expect("unspent-output cohort source"),
                Version::TWO,
                mappings,
                window_starts,
            )
        });
        Ok(Self { cohorts, stored })
    }
}
