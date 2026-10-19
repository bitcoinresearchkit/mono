use bitview_plugin_indexer::Indexer;
use bitview_primitives::{Index40, TxInIndex, TxOutIndex};
use bitview_traversable::Traversable;
use brk_types::{TxIndex, Txid, Version};
use vecdb::{IndexVec, ReadableBoxedVec, ReadableCloneableVec};

use crate::LazyIndexCountVec;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Global zero-based transaction index in canonical blockchain order. At `tx_index`,
    /// returns the transaction index itself.
    pub identity: IndexVec<TxIndex, TxIndex, ReadableBoxedVec<TxIndex, Txid>>,
    /// Number of inputs in the indexed transaction, including the coinbase
    /// transaction's single input.
    pub input_count: LazyIndexCountVec<TxIndex, TxInIndex>,
    /// Number of outputs in the indexed transaction.
    pub output_count: LazyIndexCountVec<TxIndex, Index40<TxOutIndex>>,
}

impl Vecs {
    pub fn new(version: Version, indexer: &Indexer) -> Self {
        Self {
            identity: IndexVec::new(
                "tx_index",
                version,
                indexer.vecs().transactions.txid.read_only_boxed_clone(),
                |index| index,
            ),
            input_count: LazyIndexCountVec::new(
                "tx_input_count",
                version,
                &indexer.vecs().transactions.first_txin_index,
                &*indexer.vecs().inputs.outpoint,
            ),
            output_count: LazyIndexCountVec::new(
                "tx_output_count",
                version,
                &*indexer.vecs().transactions.first_txout_index,
                &indexer.vecs().outputs.value,
            ),
        }
    }
}
