use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::{Mutex, RwLock};

use crate::{Database, database::weak::WeakDatabase, dirty_ranges::DirtyRanges};

use super::RegionMetadata;

#[derive(Debug)]
pub(crate) struct RegionInner {
    pub(crate) db: WeakDatabase,
    pub(crate) index: usize,
    accessed: AtomicBool,
    pub(crate) meta: RwLock<RegionMetadata>,
    /// Readers hold this for the lifetime of their bytes; mutations hold it exclusively.
    pub(crate) access: RwLock<()>,
    pub(crate) tail_needs_punch: AtomicBool,
    /// Sorted, merged dirty byte ranges relative to the region start.
    dirty_ranges: Mutex<DirtyRanges>,
}

impl RegionInner {
    pub(super) fn new(db: &Database, index: usize, meta: RegionMetadata) -> Self {
        Self {
            db: WeakDatabase::new(db),
            index,
            accessed: AtomicBool::new(false),
            meta: RwLock::new(meta),
            access: RwLock::new(()),
            tail_needs_punch: AtomicBool::new(false),
            dirty_ranges: Mutex::new(DirtyRanges::default()),
        }
    }

    pub(crate) fn mark_accessed(&self) {
        self.accessed.store(true, Ordering::Relaxed);
    }

    pub(crate) fn was_accessed(&self) -> bool {
        self.accessed.load(Ordering::Relaxed)
    }

    #[inline]
    pub(crate) fn mark_dirty(&self, offset: usize, len: usize) {
        self.dirty_ranges.lock().insert(offset..offset + len);
    }

    /// Appends absolute ranges without changing dirty state. Returns whether
    /// this region has dirty data. The caller holds the database mutation barrier.
    pub(crate) fn append_dirty_ranges(&self, ranges: &mut DirtyRanges) -> bool {
        let dirty = self.dirty_ranges.lock();
        if dirty.is_empty() {
            return false;
        }
        let start = self.meta.read().start();
        ranges.extend(
            dirty
                .iter()
                .map(|range| start + range.start..start + range.end),
        );
        true
    }

    /// Called only after a successful flush, while mutations are still excluded.
    pub(crate) fn clear_dirty_ranges(&self) {
        *self.dirty_ranges.lock() = DirtyRanges::default();
    }
}
