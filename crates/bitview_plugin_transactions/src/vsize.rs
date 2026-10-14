use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::ops::Convert;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{TxIndex, VSize, Version, Weight32};
use vecdb::{Database, LazyVec, ReadableCloneableVec, Rw, StorageMode};

use crate::distribution::TxDistribution;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub tx: LazyVec<TxIndex, VSize, TxIndex, Weight32>,
    #[traversable(flatten)]
    distribution: TxDistribution<VSize, M>,
}

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        Ok(Self {
            tx: LazyVec::transformed::<Convert>(
                "tx_vsize",
                version,
                indexer.vecs().transactions.weight.read_only_boxed_clone(),
            ),
            distribution: TxDistribution::import(db, "tx_vsize", version, mappings)?,
        })
    }

    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        exit: &Exit,
    ) -> Result<()> {
        self.distribution.compute(
            mappings,
            &indexer.safe_lengths(),
            &indexer.vecs().transactions.first_tx_index,
            &self.tx,
            0,
            exit,
        )
    }
}
