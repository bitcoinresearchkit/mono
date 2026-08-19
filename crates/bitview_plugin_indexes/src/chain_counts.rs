use brk_indexer::Indexer;
use brk_types::{Height, StoredU64, TxInIndex, TxIndex, TxOutIndex, Version};
use vecdb::{CachedBoxedVec, CachedReadableVec, CachedVec, ReadableCloneableVec};

use bitview_compute::LazyCumulativeIndexVec;

/// Pinned canonical cumulative counts derived from the indexer's first-index
/// boundaries.
#[derive(Clone)]
pub struct CachedChainCounts {
    transaction: CachedVec<LazyCumulativeIndexVec<Height, TxIndex>>,
    input: CachedVec<LazyCumulativeIndexVec<Height, TxInIndex>>,
    output: CachedVec<LazyCumulativeIndexVec<Height, TxOutIndex>>,
}

impl CachedChainCounts {
    pub fn new(version: Version, indexer: &Indexer) -> Self {
        Self {
            transaction: CachedVec::wrap(LazyCumulativeIndexVec::new(
                "tx_count_cumulative",
                version,
                indexer
                    .vecs()
                    .transactions
                    .first_tx_index
                    .read_only_boxed_clone(),
                indexer.vecs().transactions.txid.read_only_boxed_clone(),
            )),
            input: CachedVec::wrap(LazyCumulativeIndexVec::new(
                "input_count_cumulative",
                version,
                indexer
                    .vecs()
                    .inputs
                    .first_txin_index
                    .read_only_boxed_clone(),
                indexer.vecs().inputs.outpoint.read_only_boxed_clone(),
            )),
            output: CachedVec::wrap(LazyCumulativeIndexVec::new(
                "output_count_cumulative",
                version,
                indexer
                    .vecs()
                    .outputs
                    .first_txout_index
                    .read_only_boxed_clone(),
                indexer.vecs().outputs.value.read_only_boxed_clone(),
            )),
        }
    }

    pub fn transaction_source(&self) -> CachedVec<LazyCumulativeIndexVec<Height, TxIndex>> {
        self.transaction.clone()
    }

    pub fn input_source(&self) -> CachedVec<LazyCumulativeIndexVec<Height, TxInIndex>> {
        self.input.clone()
    }

    pub fn output_source(&self) -> CachedVec<LazyCumulativeIndexVec<Height, TxOutIndex>> {
        self.output.clone()
    }

    pub fn output(&self) -> CachedBoxedVec<Height, StoredU64> {
        self.output.cached_boxed_clone()
    }

    pub fn invalidate(&self) {
        self.transaction.invalidate();
        self.input.invalidate();
        self.output.invalidate();
    }
}
