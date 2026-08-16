use brk_indexer::Indexer;
use brk_traversable::Traversable;
use brk_types::{OutPoint, TxInIndex, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Global zero-based transaction-input index in canonical blockchain order.
    /// At `txin_index`, this is the identity value; at `txout_index`, it
    /// identifies the input that spends the output, with `u64::MAX` representing
    /// an unspent output.
    pub identity: LazyVec<TxInIndex, TxInIndex, TxInIndex, OutPoint>,
}

impl Vecs {
    pub(crate) fn forced_import(version: Version, indexer: &Indexer) -> Self {
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
