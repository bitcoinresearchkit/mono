use brk_cohort::AmountRange;
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{StoredU64, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use crate::{
    distribution::metrics::UTXORows,
    indexes,
    internal::{CachedWindowStartVec, Windows},
};

use super::{SpentOutputCount, UnspentOutputCount};

#[derive(Traversable)]
pub struct OutputsVecs<M: StorageMode = Rw> {
    pub unspent_count: UnspentOutputCount<M>,
    pub spent_count: SpentOutputCount<M>,
}

impl OutputsVecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            unspent_count: UnspentOutputCount::forced_import(db, version, indexes, cached_starts)?,
            spent_count: SpentOutputCount::forced_import(db, version, indexes, cached_starts)?,
        })
    }

    #[inline(always)]
    pub(crate) fn push(
        &mut self,
        unspent_count: UTXORows<StoredU64>,
        spent_count: UTXORows<StoredU64>,
    ) {
        self.unspent_count.matrices.push(unspent_count);
        self.spent_count.cumulative.push_block(spent_count);
    }

    #[inline(always)]
    pub(crate) fn push_addr_balance(&mut self, row: AmountRange<StoredU64>) {
        self.unspent_count.push_addr_balance(row);
    }

    pub(crate) fn min_len(&self) -> usize {
        self.unspent_count
            .matrices
            .min_len()
            .min(self.unspent_count.addr_balance.len())
            .min(self.spent_count.cumulative.min_len())
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.unspent_count.matrices.collect_vecs_mut();
        vecs.push(self.unspent_count.addr_balance.stored_mut());
        vecs.extend(self.spent_count.cumulative.collect_vecs_mut());
        vecs
    }
}
