use bitview_plugin_indexer::Indexer;
use bitview_primitives::TxOutIndex;
use bitview_traversable::Traversable;
use brk_types::{OutputType, Version};
use vecdb::{IndexVec, ReadableBoxedVec, ReadableCloneableVec};

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Global zero-based transaction-output index in canonical blockchain order. At
    /// `txout_index`, returns the output index itself.
    pub identity: IndexVec<TxOutIndex, TxOutIndex, ReadableBoxedVec<TxOutIndex, OutputType>>,
}

impl Vecs {
    pub fn new(version: Version, indexer: &Indexer) -> Self {
        Self {
            identity: IndexVec::new(
                "txout_index",
                version,
                // Any outputs column gives the length; `value` would mark this view mutable.
                indexer.vecs().outputs.output_type.read_only_boxed_clone(),
                |index| index,
            ),
        }
    }
}
