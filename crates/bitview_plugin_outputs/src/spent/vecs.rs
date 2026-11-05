use bitview_primitives::{Index40, TxInIndex, TxOutIndex};
use bitview_traversable::{Compact, Traversable};
use vecdb::{BytesVec, MutableVec, Rw, StorageMode};

/// Each output's spending input, 5 bytes per output (`TxInIndex::UNSPENT` while unspent).
pub type Spends = MutableVec<BytesVec<TxOutIndex, Index40<TxInIndex>>>;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// The transaction input that spends the output; null while the output is unspent.
    pub spending_txin_index: Compact<Spends, Option<TxInIndex>, M>,
}
