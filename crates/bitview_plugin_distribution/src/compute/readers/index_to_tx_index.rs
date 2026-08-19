use brk_types::{StoredU64, TxIndex};
use vecdb::{ReadableVec, VecIndex};

/// Reusable buffers for a block's index-to-transaction-index mapping.
pub struct IndexToTxIndexBuf {
    counts: Vec<StoredU64>,
    result: Vec<TxIndex>,
}

impl IndexToTxIndexBuf {
    pub fn new() -> Self {
        Self {
            counts: Vec::new(),
            result: Vec::new(),
        }
    }

    pub fn build(
        &mut self,
        block_first_tx_index: TxIndex,
        block_tx_count: u64,
        tx_index_to_count: &impl ReadableVec<TxIndex, StoredU64>,
    ) -> &[TxIndex] {
        let first = block_first_tx_index.to_usize();
        tx_index_to_count.collect_range_into_at(
            first,
            first + block_tx_count as usize,
            &mut self.counts,
        );

        let total: u64 = self.counts.iter().map(|count| u64::from(*count)).sum();
        self.result.clear();
        self.result.reserve(total as usize);

        for (offset, count) in self.counts.iter().enumerate() {
            let tx_index = TxIndex::from(first + offset);
            self.result
                .extend(std::iter::repeat_n(tx_index, u64::from(*count) as usize));
        }

        &self.result
    }
}
