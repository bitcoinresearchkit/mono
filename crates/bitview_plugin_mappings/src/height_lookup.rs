use std::sync::Arc;

use arc_swap::{ArcSwap, Guard};
use brk_types::{Height, TxIndex};
use vecdb::{ReadableVec, VecIndex, VecValue};

mod map;

pub use map::HeightMap;

/// Resident block boundaries, published for read-only consumers: each load returns one
/// immutable root, so lookups never wait for the updater.
pub struct HeightLookup<I>(Arc<ArcSwap<HeightMap<I>>>);

impl<I> Clone for HeightLookup<I> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<I: VecIndex> HeightLookup<I> {
    /// The latest published mapping, for a short batch of lookups.
    pub fn load(&self) -> Guard<Arc<HeightMap<I>>> {
        self.0.load()
    }

    /// The latest published mapping, owned: for a compute holding it throughout.
    pub fn load_full(&self) -> Arc<HeightMap<I>> {
        self.0.load_full()
    }

    pub(crate) fn init(source: &impl ReadableVec<Height, I>) -> Self
    where
        I: VecValue,
    {
        let this = Self(Arc::new(ArcSwap::from_pointee(HeightMap::default())));
        this.update(source, Height::ZERO);
        this
    }

    /// Extend with new blocks since last call. Truncates on reorg. Publishes a new root that
    /// shares every unchanged page with the previous one.
    pub(crate) fn update(&self, source: &impl ReadableVec<Height, I>, reorg_height: Height)
    where
        I: VecValue,
    {
        let mut map = HeightMap::clone(&self.load());
        let from = map.len().min(reorg_height.to_usize()).min(source.len());
        map.update_at(from, source.collect_range_at(from, source.len()));
        self.0.store(Arc::new(map));
    }

    /// Look up the block containing an index.
    #[inline]
    pub fn get(&self, index: I) -> Option<Height> {
        self.load().get(index)
    }
}

impl HeightLookup<TxIndex> {
    /// Resume at the block containing the next transaction, or the height end
    /// when every transaction in the requested range is already computed.
    pub fn resume_height(&self, tx_len: usize, target_tx: usize, target_height: usize) -> usize {
        self.load().resume_height(tx_len, target_tx, target_height)
    }
}
