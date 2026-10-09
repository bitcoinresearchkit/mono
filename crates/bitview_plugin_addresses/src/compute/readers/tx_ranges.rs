use std::iter;

use bitview_primitives::Count;
use brk_types::TxIndex;
use vecdb::{ReadableVec, VecIndex};

/// Each entry's transaction index for a block's transactions, from their entry counts.
pub fn tx_indexes(
    block_first_tx_index: TxIndex,
    block_tx_count: u64,
    tx_index_to_count: &impl ReadableVec<TxIndex, Count>,
) -> impl Iterator<Item = TxIndex> {
    let first = block_first_tx_index.to_usize();
    tx_index_to_count
        .collect_range_at(first, first + block_tx_count as usize)
        .into_iter()
        .enumerate()
        .flat_map(move |(offset, count)| {
            iter::repeat_n(TxIndex::from(first + offset), u64::from(count) as usize)
        })
}
