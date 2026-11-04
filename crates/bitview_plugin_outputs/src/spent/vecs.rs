use std::hint::black_box;

use bitview_primitives::{Index40, TxInIndex, TxOutIndex};
use bitview_traversable::{Compact, Traversable};
use rayon::prelude::*;
use vecdb::{BytesVec, MutableVec, Rw, StorageMode};

/// Each output's spending input, 5 bytes per output (`TxInIndex::UNSPENT` while unspent).
pub type Spends = MutableVec<BytesVec<TxOutIndex, Index40<TxInIndex>>>;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// The transaction input that spends the output; null while the output is unspent.
    pub spending_txin_index: Compact<Spends, Option<TxInIndex>, M>,
}

impl Vecs {
    /// Reads `spent` outputs' entries, which the next update's spends rewrite, keeping nothing.
    pub fn warm(&self, spent: &[TxOutIndex]) {
        spent.par_iter().for_each_init(
            || self.spending_txin_index.reader(),
            |reader, &txout_index| {
                black_box(reader.try_get(txout_index));
            },
        );
    }
}
