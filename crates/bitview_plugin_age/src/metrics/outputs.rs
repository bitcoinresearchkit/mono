use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_distribution::families::{CountWithDeltas, CumulativeCount};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct OutputsVecs<M: StorageMode = Rw> {
    /// Number of transaction outputs that are unspent at the represented block.
    pub unspent_count: CountWithDeltas<M>,
    /// Number of the cohort's outputs spent in each block.
    pub spent_count: CumulativeCount<Count, M>,
}

impl OutputsVecs {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Utxo.metric_name(cohort, metric);
        Ok(Self {
            unspent_count: CountWithDeltas::import(
                db,
                &name("utxo_count"),
                version,
                mappings,
                window_starts,
            )?,
            spent_count: CumulativeCount::import(
                db,
                &name("spent_utxo_count"),
                version + Version::ONE,
                mappings,
                window_starts,
            )?,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, (unspent, spent): (Count, Count)) {
        self.unspent_count.push(unspent);
        self.spent_count.push_block(spent);
    }

    pub fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 2] {
        [
            self.unspent_count.stored_mut(),
            self.spent_count.stored_mut(),
        ]
    }
}
