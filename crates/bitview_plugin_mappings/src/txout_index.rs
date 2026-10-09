use bitview_plugin_indexer::Indexer;
use bitview_primitives::TxOutIndex;
use bitview_traversable::Traversable;
use brk_types::{OutputType, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Global zero-based transaction-output index in canonical blockchain order.
    /// At `txout_index`, this is the identity value; at `txin_index`, it
    /// identifies the previous output spent by the input, with `u64::MAX`
    /// representing a coinbase input.
    pub identity: LazyVec<TxOutIndex, TxOutIndex, TxOutIndex, OutputType>,
}

impl Vecs {
    pub fn new(version: Version, indexer: &Indexer) -> Self {
        Self {
            identity: LazyVec::init(
                "txout_index",
                version,
                // Any outputs column gives the length; `value` would mark this view mutable.
                indexer.vecs().outputs.output_type.read_only_boxed_clone(),
                |index, _| index,
            ),
        }
    }
}
