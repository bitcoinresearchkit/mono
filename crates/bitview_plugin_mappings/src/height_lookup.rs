use std::sync::Arc;

use brk_types::{Height, TxIndex};
use parking_lot::{RwLock, RwLockReadGuard};
use vecdb::{ReadableVec, VecIndex, VecValue};

mod map;

pub use map::HeightMap;

/// Resident block boundaries shared by the updater and read-only consumers.
/// Clones share one map; read guards pin a consistent suffix during lookups.
#[derive(Clone)]
pub struct HeightLookup<I>(Arc<RwLock<HeightMap<I>>>);

impl<I: VecIndex> HeightLookup<I> {
    /// Hold a stable mapping for a batch of lookups, with one read lock.
    pub fn read(&self) -> RwLockReadGuard<'_, HeightMap<I>> {
        self.0.read_recursive()
    }

    pub(crate) fn init(source: &impl ReadableVec<Height, I>) -> Self
    where
        I: VecValue,
    {
        Self(Arc::new(RwLock::new(HeightMap::from(source.collect()))))
    }

    /// Extend with new blocks since last call. Truncates on reorg.
    pub(crate) fn update(&self, source: &impl ReadableVec<Height, I>, reorg_height: Height)
    where
        I: VecValue,
    {
        let from = self
            .read()
            .len()
            .min(reorg_height.to_usize())
            .min(source.len());
        let starts = source.collect_range_at(from, source.len());
        self.0.write().update_at(from, starts);
    }

    /// Look up the block containing an index.
    #[inline]
    pub fn get_shared(&self, index: I) -> Option<Height> {
        self.0.read_recursive().get_shared(index)
    }
}

impl HeightLookup<TxIndex> {
    /// Resume at the block containing the next transaction, or the height end
    /// when every transaction in the requested range is already computed.
    pub fn resume_height(&self, tx_len: usize, target_tx: usize, target_height: usize) -> usize {
        self.read().resume_height(tx_len, target_tx, target_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_share_directory_updates_and_reorgs() {
        let heights = HeightLookup(Arc::new(RwLock::new(HeightMap::from(
            [0, 65_536, 131_072].map(TxIndex::new).to_vec(),
        ))));
        let reader = heights.clone();
        assert_eq!(
            reader.get_shared(TxIndex::new(100_000)),
            Some(Height::new(1))
        );
        heights.0.write().update_at(1, [TxIndex::new(200_000)]);
        assert_eq!(reader.get_shared(TxIndex::new(100_000)), Some(Height::ZERO));
        assert_eq!(
            reader.get_shared(TxIndex::new(200_000)),
            Some(Height::new(1))
        );
        // Ordered query cursors still observe the same boundary source.
        let map = reader.read();
        let mut cursor = map.cursor();
        for index in [0, 100_000, 200_000, u32::MAX] {
            assert_eq!(
                cursor.get(TxIndex::new(index)),
                map.get_shared(TxIndex::new(index))
            );
        }
    }

    #[test]
    fn resume_height_includes_partial_blocks_and_stops_at_the_target() {
        let heights = HeightLookup(Arc::new(RwLock::new(HeightMap::from(
            [0usize, 1, 4].map(TxIndex::from).to_vec(),
        ))));
        for (tx_len, expected) in [
            (0, 0),
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 2),
            (5, 2),
            (6, 3),
            (7, 3),
        ] {
            assert_eq!(heights.resume_height(tx_len, 6, 3), expected);
        }
        assert_eq!(heights.resume_height(4, 4, 2), 2);
        assert_eq!(heights.resume_height(0, 0, 0), 0);
    }
}
