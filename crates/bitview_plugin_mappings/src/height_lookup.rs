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
