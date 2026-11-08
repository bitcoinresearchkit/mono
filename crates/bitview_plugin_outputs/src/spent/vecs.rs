use bitview_primitives::{Index40, Lengths, TxInIndex, TxOutIndex};
use bitview_traversable::{Compact, Traversable};
use vecdb::{BytesVec, MutableVec, Rw, StorageMode};

/// Each output's spending input, 5 bytes per output (`TxInIndex::UNSPENT` while unspent).
pub type Spends = MutableVec<BytesVec<TxOutIndex, Index40<TxInIndex>>>;

/// The spend a stored row records, as `safe` publishes it: spends are written in place before the
/// update that indexes their inputs publishes, so a spend past the published inputs reads as none,
/// as does the sentinel of an output no input spends.
#[inline]
pub fn published_spend(stored: Index40<TxInIndex>, safe: Lengths) -> Option<TxInIndex> {
    let spend = stored.get();
    (spend < safe.txin_index).then_some(spend)
}

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// The transaction input that spends the output; null while the output is unspent.
    pub spending_txin_index: Compact<Spends, Option<TxInIndex>, M>,
}
