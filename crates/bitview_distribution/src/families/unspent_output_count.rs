use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, CountSigned, PartsPerMillionSigned64};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlockWithDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

use super::import_stored;

/// One cohort's unspent output count, stored as `utxo_count`.
#[derive(Traversable)]
pub struct UnspentOutputCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    count: LazyPerBlockWithDeltas<Count, CountSigned, PartsPerMillionSigned64>,
    #[traversable(hidden)]
    pub stored: CachedSeries<Height, Count, M>,
}

impl UnspentOutputCount {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = import_stored(db, cohort, "utxo_count", version)?;
        let count = LazyPerBlockWithDeltas::from_height_source(
            &CohortContext::Utxo.metric_name(cohort, "utxo_count"),
            version,
            &stored,
            Version::TWO,
            mappings,
            window_starts,
        );
        Ok(Self { count, stored })
    }

    #[inline(always)]
    pub fn push(&mut self, count: Count) {
        self.stored.push(count);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.stored
    }
}
