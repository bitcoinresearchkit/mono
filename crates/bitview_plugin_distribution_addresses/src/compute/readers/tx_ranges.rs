use std::iter;

use bitview_primitives::StoredU64;
use brk_types::TxIndex;
use vecdb::{ReadableVec, VecIndex};

/// Reuse transaction counts and walk their entries without expanding a mapping.
#[derive(Default)]
pub struct TxRanges {
    counts: Vec<StoredU64>,
}

impl TxRanges {
    pub fn build(
        &mut self,
        block_first_tx_index: TxIndex,
        block_tx_count: u64,
        tx_index_to_count: &impl ReadableVec<TxIndex, StoredU64>,
    ) -> impl Iterator<Item = TxIndex> + Send + '_ {
        let first = block_first_tx_index.to_usize();
        tx_index_to_count.collect_range_into_at(
            first,
            first + block_tx_count as usize,
            &mut self.counts,
        );

        self.counts
            .iter()
            .enumerate()
            .flat_map(move |(offset, count)| {
                iter::repeat_n(TxIndex::from(first + offset), u64::from(*count) as usize)
            })
    }
}
