use bitview_plugin_indexer::Indexer;
use bitview_primitives::{TxInIndex, TxOutIndex};
use brk_types::{Height, TxIndex, Version};

use crate::LazyCumulativeIndexVec;

/// Canonical cumulative counts over the indexer's cached stored boundaries.
pub struct ChainCounts {
    pub(crate) transaction: LazyCumulativeIndexVec<Height, TxIndex>,
    pub(crate) input: LazyCumulativeIndexVec<Height, TxInIndex>,
    pub(crate) output: LazyCumulativeIndexVec<Height, TxOutIndex>,
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
                &*indexer.vecs().inputs.outpoint,
            ),
            output: LazyCumulativeIndexVec::new(
                "output_count_cumulative",
                version,
                &indexer.vecs().outputs.first_txout_index,
                &indexer.vecs().outputs.value,
            ),
        }
    }
}
