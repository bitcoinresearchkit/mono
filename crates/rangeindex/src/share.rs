use std::sync::Arc;

use arc_swap::{ArcSwap, Guard};

use crate::RangeMap;

/// Readers' handle to a published range map. Each load returns one immutable root that never
/// changes, so a read never waits for the updater and sees one consistent mapping throughout.
pub struct SharedRangeMap<I, V>(Arc<ArcSwap<RangeMap<I, V>>>);

impl<I, V> Clone for SharedRangeMap<I, V> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<I, V> SharedRangeMap<I, V> {
    /// The latest published root; load it once per read.
    #[inline]
    pub fn load(&self) -> Guard<Arc<RangeMap<I, V>>> {
        self.0.load()
    }
}

/// The updater of a [`SharedRangeMap`]: publishes a new root per update, sharing every page
/// the update leaves unchanged.
pub struct RangeMapWriter<I, V>(SharedRangeMap<I, V>);

impl<I, V> Clone for RangeMapWriter<I, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<I, V> Default for RangeMapWriter<I, V> {
    fn default() -> Self {
        Self(SharedRangeMap(Arc::new(ArcSwap::from_pointee(
            RangeMap::default(),
        ))))
    }
}

impl<I, V> RangeMapWriter<I, V> {
    /// The read-only handle consumers hold.
    pub fn shared(&self) -> SharedRangeMap<I, V> {
        self.0.clone()
    }
}

impl<I: Ord + Copy, V> RangeMapWriter<I, V> {
    /// Replace only the affected suffix. Callers must serialize updates and
    /// publish related mappings together before readers can observe them.
    pub fn update_at(&self, from: usize, first_indexes: impl IntoIterator<Item = I>) {
        let mut map = RangeMap::clone(&self.0.load());
        debug_assert!(
            from <= map.len(),
            "range map update must preserve a valid prefix"
        );
        map.truncate(from);
        map.extend(first_indexes);
        self.0.0.store(Arc::new(map));
    }
}
