use bitview_cohort::UtxoGroups;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use super::{SpentOutputCount, UnspentOutputCount};
use crate::metrics::outputs::AvgAmount;

#[derive(Traversable)]
pub struct OutputsVecs<M: StorageMode = Rw> {
    pub avg_amount: AvgAmount<M>,
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
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            avg_amount: AvgAmount::import(db, version, mappings, spot)?,
            unspent_count: UnspentOutputCount::import(db, version, mappings, window_starts)?,
            spent_count: SpentOutputCount::import(db, version, mappings, window_starts)?,
        }))
    }

    #[inline(always)]
    pub fn push(&mut self, unspent_count: UtxoGroups<Count>, spent_count: UtxoGroups<Count>) {
        self.unspent_count.stored.push(&unspent_count);
        self.spent_count.stored.push_block(spent_count);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.unspent_count
            .stored
            .stored_vecs_mut()
            .chain(self.spent_count.stored.stored_vecs_mut())
            .chain(self.avg_amount.stored_vecs_mut())
    }
}
