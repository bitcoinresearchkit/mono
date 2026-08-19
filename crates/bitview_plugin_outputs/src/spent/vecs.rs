use bitview_traversable::Traversable;
use brk_types::{TxInIndex, TxOutIndex};
use vecdb::{BytesVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Global zero-based transaction-input index in canonical blockchain order.
    /// At `txin_index`, this is the identity value; at `txout_index`, it
    /// identifies the input that spends the output, with `u64::MAX` representing
    /// an unspent output.
    pub txin_index: M::Stored<BytesVec<TxOutIndex, TxInIndex>>,
}
