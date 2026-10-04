use bitview_plugin_indexer::Indexer;
use bitview_primitives::{Count, TxInIndex, TxOutIndex};
use brk_types::{Height, TxIndex, Version};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec};

use crate::LazyCumulativeIndexVec;

/// Canonical cumulative counts over the indexer's cached stored boundaries.
#[derive(Clone)]
pub struct ChainCounts {
    transaction: LazyCumulativeIndexVec<Height, TxIndex>,
    input: LazyCumulativeIndexVec<Height, TxInIndex>,
    output: LazyCumulativeIndexVec<Height, TxOutIndex>,
}

impl ChainCounts {
    pub fn new(version: Version, indexer: &Indexer) -> Self {
        Self {
            transaction: LazyCumulativeIndexVec::new(
                "tx_count_cumulative",
                version,
                &indexer.vecs().transactions.first_tx_index,
                &indexer.vecs().transactions.txid,
            ),
            input: LazyCumulativeIndexVec::new(
                "input_count_cumulative",
                version,
                &indexer.vecs().inputs.first_txin_index,
                &indexer.vecs().inputs.outpoint,
            ),
            output: LazyCumulativeIndexVec::new(
                "output_count_cumulative",
                version,
                &indexer.vecs().outputs.first_txout_index,
                &indexer.vecs().outputs.value,
            ),
        }
    }

    pub fn transaction_source(&self) -> LazyCumulativeIndexVec<Height, TxIndex> {
        self.transaction.clone()
    }

    pub fn input_source(&self) -> LazyCumulativeIndexVec<Height, TxInIndex> {
        self.input.clone()
    }

    pub fn output_source(&self) -> LazyCumulativeIndexVec<Height, TxOutIndex> {
        self.output.clone()
    }

    pub fn output(&self) -> ReadableBoxedVec<Height, Count> {
        self.output.read_only_boxed_clone()
    }
}
