use bitview_primitives::{Index40, TxInIndex, TxOutIndex};
use bitview_traversable::Traversable;
use vecdb::{BytesVec, LazyVec, MutableVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Stored in 5 bytes per output; series readers see `txin_index_view`.
    #[traversable(hidden)]
    pub txin_index: M::Stored<MutableVec<BytesVec<TxOutIndex, Index40<TxInIndex>>>>,
    /// The transaction input that spends the output, `u64::MAX` while the output is
    /// unspent.
    #[traversable(rename = "txin_index")]
    pub txin_index_view: LazyVec<TxOutIndex, TxInIndex, TxOutIndex, Index40<TxInIndex>>,
}
