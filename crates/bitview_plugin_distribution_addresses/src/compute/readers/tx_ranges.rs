use std::iter;

use brk_types::{StoredU64, TxIndex};
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, Version, WritableVec};

    #[test]
    fn walks_empty_transactions_coinbase_and_reused_block_ranges() {
        crate::test_cache::init_cache();
        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let mut counts = PcoVec::<TxIndex, StoredU64>::import(&db, "counts", Version::ONE).unwrap();
        for count in [1u64, 0, 2, 3, 0, 1] {
            counts.push(StoredU64::from(count));
        }
        counts.write().unwrap();
        let mut ranges = TxRanges::default();
        assert_eq!(
            ranges
                .build(TxIndex::ZERO, 3, &counts)
                .map(usize::from)
                .collect::<Vec<_>>(),
            [0, 2, 2]
        );
        assert_eq!(
            ranges
                .build(TxIndex::from(3usize), 3, &counts)
                .map(usize::from)
                .collect::<Vec<_>>(),
            [3, 3, 3, 5]
        );
        // The input loop skips exactly the coinbase input, even with empty transactions.
        assert_eq!(
            ranges
                .build(TxIndex::ZERO, 3, &counts)
                .skip(1)
                .map(usize::from)
                .collect::<Vec<_>>(),
            [2, 2]
        );
        assert_eq!(ranges.build(TxIndex::from(1usize), 1, &counts).count(), 0);
        assert_eq!(ranges.build(TxIndex::from(6usize), 0, &counts).count(), 0);
    }
}
