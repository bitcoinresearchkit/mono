use bitview_plugin_indexer::Indexer;
use bitview_primitives::TxInIndex;
use bitview_traversable::Traversable;
use brk_types::{OutPoint, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Global zero-based transaction-input index in canonical blockchain order. At `txin_index`,
    /// returns the input index itself.
    pub identity: LazyVec<TxInIndex, TxInIndex, TxInIndex, OutPoint>,
}

impl Vecs {
    pub fn new(version: Version, indexer: &Indexer) -> Self {
        Self {
            identity: LazyVec::init(
                "txin_index",
                version,
                indexer.vecs().inputs.outpoint.read_only_boxed_clone(),
                |index, _| index,
            ),
        }
    }
}
