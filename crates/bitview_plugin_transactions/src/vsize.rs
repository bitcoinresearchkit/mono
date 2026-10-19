use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{TxIndex, VSize, Version, Weight32};
use vecdb::{Convert, Database, LazyVec, ReadableCloneableVec, Rw, StorageMode};

use crate::distribution::TxDistribution;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub tx: LazyVec<TxIndex, VSize, Weight32>,
    // Filled by the fee pass, from the vsizes it reads.
    #[traversable(flatten)]
    pub(crate) distribution: TxDistribution<VSize, M>,
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
}
