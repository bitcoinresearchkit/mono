use bitview_primitives::{Index40, TxInIndex, TxOutIndex};
use bitview_traversable::{Compact, Traversable};
use vecdb::{BytesVec, MutableVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// The transaction input that spends the output, `u64::MAX` while the output is
    /// unspent.
    // Stored in 5 bytes per output.
    pub txin_index: Compact<MutableVec<BytesVec<TxOutIndex, Index40<TxInIndex>>>, TxInIndex, M>,
}
