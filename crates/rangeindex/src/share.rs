use std::sync::Arc;

use parking_lot::{RwLock, RwLockReadGuard};

use crate::RangeMap;

/// One resident range map shared by its updater and readers.
/// Clones share the same allocation; a read guard pins a consistent mapping.
#[derive(Clone)]
pub struct SharedRangeMap<I, V>(Arc<RwLock<RangeMap<I, V>>>);

impl<I: Default + Copy, V: Default + Copy> SharedRangeMap<I, V> {
    pub fn new(first_indexes: Vec<I>) -> Self {
        Self(Arc::new(RwLock::new(RangeMap::from(first_indexes))))
    }
}

impl<I, V> SharedRangeMap<I, V> {
    pub fn read(&self) -> RwLockReadGuard<'_, RangeMap<I, V>> {
        self.0.read_recursive()
    }
}

impl<I: Ord + Copy, V> SharedRangeMap<I, V> {
    /// Replace only the affected suffix. Callers must serialize updates and
    /// publish related mappings together before readers can observe them.
    pub fn update_at(&self, from: usize, first_indexes: impl IntoIterator<Item = I>) {
        let mut map = self.0.write();
        assert!(
            from <= map.len(),
            "range map update must preserve a valid prefix"
        );
        map.truncate(from);
        map.extend(first_indexes);
    }
}
