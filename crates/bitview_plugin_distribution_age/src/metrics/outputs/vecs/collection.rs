use bitview_cohort::CreationCohorts;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{StoredU64, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::{SpentOutputCount, UnspentOutputCount};

#[derive(Traversable)]
pub struct OutputsVecs<M: StorageMode = Rw> {
    /// Number of transaction outputs that are unspent at the represented block.
    pub unspent_count: UnspentOutputCount<M>,
    /// Number of outputs from a UTXO cohort spent in each block.
    pub spent_count: SpentOutputCount<M>,
}

impl OutputsVecs {
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            unspent_count: UnspentOutputCount::import(db, version, mappings, window_starts)?,
            spent_count: SpentOutputCount::import(db, version, mappings, window_starts)?,
        }))
    }

    #[inline(always)]
    pub fn push(
        &mut self,
        unspent_count: CreationCohorts<StoredU64>,
        spent_count: CreationCohorts<StoredU64>,
    ) {
        self.unspent_count.stored.push(&unspent_count);
        self.spent_count.stored.push_block(spent_count);
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self
            .unspent_count
            .stored
            .stored_vecs_mut()
            .collect::<Vec<_>>();
        vecs.extend(self.spent_count.stored.stored_vecs_mut());
        vecs
    }
}
