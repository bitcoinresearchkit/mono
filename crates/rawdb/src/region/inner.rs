use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

use parking_lot::RwLock;

use crate::{Database, PAGE_SIZE, database::weak::WeakDatabase, dirty_ranges::DirtyRanges};

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
    dirty_ranges: UnsafeCell<DirtyRanges>,
}

// SAFETY: dirty ranges are mutated only with exclusive region access plus the
// mutation barrier. Flush reads/clears them with that barrier held exclusively.
// All remaining shared fields are immutable, atomic, or independently locked.
unsafe impl Sync for RegionInner {}

impl RegionInner {
    pub(super) fn new(db: &Database, index: usize, meta: RegionMetadata) -> Self {
        Self {
            db: WeakDatabase::new(db),
            index,
            accessed: AtomicBool::new(false),
            meta: RwLock::new(meta),
            access: RwLock::new(()),
            tail_needs_punch: AtomicBool::new(false),
            dirty_ranges: UnsafeCell::new(DirtyRanges::default()),
        }
    }

    /// # Safety
    /// Hold region access or the exclusive database mutation barrier.
    /// Every metadata mutation takes both region access and the shared barrier.
    pub(crate) unsafe fn bounds(&self) -> (usize, usize, usize) {
        let meta = unsafe { &*self.meta.data_ptr() };
        (meta.start(), meta.len(), meta.reserved())
    }

    pub(crate) fn mark_accessed(&self) {
        self.accessed.store(true, Ordering::Relaxed);
    }

    pub(crate) fn was_accessed(&self) -> bool {
        self.accessed.load(Ordering::Relaxed)
    }

    /// # Safety
    /// Hold exclusive region access and the database mutation barrier.
    #[inline]
    pub(crate) unsafe fn mark_dirty(&self, offset: usize, len: usize) {
        unsafe { &mut *self.dirty_ranges.get() }.insert(offset..offset + len);
    }

    /// Appends page-covered absolute ranges without changing dirty state.
    ///
    /// # Safety
    /// Hold the database mutation barrier exclusively.
    pub(crate) unsafe fn append_dirty_ranges(&self, ranges: &mut DirtyRanges) -> bool {
        // The barrier excludes every writer; another per-region lock is redundant.
        let dirty = unsafe { &*self.dirty_ranges.get() };
        if dirty.is_empty() {
            return false;
        }
        let (start, _, _) = unsafe { self.bounds() };
        // Allocations have PAGE_SIZE-aligned bounds, so rounding stays within
        // reserved storage. Merge flushes that cover the same or adjacent pages
        // while keeping each region's dirty byte ranges unchanged until success.
        ranges.extend(dirty.iter().map(|range| {
            let first = start + range.start;
            let end = start + range.end;
            first / PAGE_SIZE * PAGE_SIZE..end.next_multiple_of(PAGE_SIZE)
        }));
        true
    }

    /// # Safety
    /// Hold the mutation barrier exclusively; both files have synchronized.
    pub(crate) unsafe fn clear_dirty_ranges(&self) {
        unsafe { *self.dirty_ranges.get() = DirtyRanges::default() };
    }
}
