use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CumulativeRolling, LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{TxIndex, VSize, Version, Weight32};
use vecdb::{Convert, Database, LazyVec, ReadableCloneableVec, Rw, StorageMode};

use crate::distribution::TxDistribution;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(accumulation = Flow)]
    pub tx: LazyVec<TxIndex, VSize, Weight32>,
    // Filled by the fee pass, from the vsizes it reads.
    #[traversable(flatten)]
    pub(crate) distribution: TxDistribution<VSize, M>,
    /// Every transaction's virtual size added up, coinbase included.
    #[traversable(flatten)]
    pub(crate) total: CumulativeRolling<VSize, M>,
}

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            tx: LazyVec::transformed::<Convert>(
                "tx_vsize",
                version,
                indexer.vecs().transactions.weight.read_only_boxed_clone(),
            ),
            distribution: TxDistribution::import(db, "tx_vsize", version, mappings)?,
            total: CumulativeRolling::import(db, "tx_vsize", version, mappings, window_starts)?,
        })
    }
}
