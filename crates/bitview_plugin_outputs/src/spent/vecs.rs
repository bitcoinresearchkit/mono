use bitview_primitives::{Index40, TxInIndex, TxOutIndex};
use bitview_traversable::Traversable;
use vecdb::{BytesVec, LazyVec, MutableVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Stored in 5 bytes per output; series readers see `txin_index_view`.
    #[traversable(hidden)]
    pub txin_index: M::Stored<MutableVec<BytesVec<TxOutIndex, Index40<TxInIndex>>>>,
    /// Global zero-based transaction-input index in canonical blockchain order.
    /// At `txin_index`, this is the identity value; at `txout_index`, it
    /// identifies the input that spends the output, with `u64::MAX` representing
    /// an unspent output.
    #[traversable(rename = "txin_index")]
    pub txin_index_view: LazyVec<TxOutIndex, TxInIndex, TxOutIndex, Index40<TxInIndex>>,
}
