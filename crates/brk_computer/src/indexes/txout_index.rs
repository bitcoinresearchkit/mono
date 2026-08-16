use brk_indexer::Indexer;
use brk_traversable::Traversable;
use brk_types::{Sats, TxOutIndex, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Global zero-based transaction-output index in canonical blockchain order.
    /// At `txout_index`, this is the identity value; at `txin_index`, it
    /// identifies the previous output spent by the input, with `u64::MAX`
    /// representing a coinbase input.
    pub identity: LazyVec<TxOutIndex, TxOutIndex, TxOutIndex, Sats>,
}

impl Vecs {
    pub(crate) fn forced_import(version: Version, indexer: &Indexer) -> Self {
        Self {
            identity: LazyVec::init(
                "txout_index",
                version,
                indexer.vecs().outputs.value.read_only_boxed_clone(),
                |index, _| index,
            ),
        }
    }
}
