use brk_indexer::Indexer;
use brk_traversable::Traversable;
use brk_types::{TxInIndex, TxIndex, TxOutIndex, Txid, Version};
use vecdb::{LazyVec, ReadableCloneableVec};

use crate::internal::LazyIndexCountVec;

#[derive(Clone, Traversable)]
pub struct Vecs {
    pub identity: LazyVec<TxIndex, TxIndex, TxIndex, Txid>,
    pub input_count: LazyIndexCountVec<TxIndex, TxInIndex>,
    pub output_count: LazyIndexCountVec<TxIndex, TxOutIndex>,
}

impl Vecs {
    pub(crate) fn new(version: Version, indexer: &Indexer) -> Self {
        Self {
            identity: LazyVec::init(
                "tx_index",
                version,
                indexer.vecs().transactions.txid.read_only_boxed_clone(),
                |index, _| index,
            ),
            input_count: LazyIndexCountVec::new(
                "input_count",
                version,
                indexer
                    .vecs()
                    .transactions
                    .first_txin_index
                    .read_only_boxed_clone(),
                indexer.vecs().inputs.outpoint.read_only_boxed_clone(),
            ),
            output_count: LazyIndexCountVec::new(
                "output_count",
                version,
                indexer
                    .vecs()
                    .transactions
                    .first_txout_index
                    .read_only_boxed_clone(),
                indexer.vecs().outputs.value.read_only_boxed_clone(),
            ),
        }
    }
}
