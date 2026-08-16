use std::{
    fs::File,
    mem,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use log::{debug, trace};
use parking_lot::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    Database, Error, HolePunch, PAGE_SIZE, PAGE_SIZE_MINUS_1, Reader, RegionMetadata, Result,
    WeakDatabase,
};

/// Named, dynamically-sized region within a database.
#[derive(Debug, Clone)]
#[must_use = "Region should be stored to access the data"]
pub struct Region(Arc<RegionInner>);

#[derive(Debug)]
pub(crate) struct RegionInner {
    db: WeakDatabase,
    index: usize,
    meta: RwLock<RegionMetadata>,
    /// Sorted, merged dirty byte ranges relative to the region start.
    dirty_ranges: Mutex<Vec<(usize, usize)>>,
    sparse_flush: AtomicBool,
}

impl Region {
    pub(crate) fn new(
        db: &Database,
        id: String,
        index: usize,
        start: usize,
        len: usize,
        reserved: usize,
    ) -> Self {
        Self(Arc::new(RegionInner {
            db: db.weak_clone(),
            index,
            meta: RwLock::new(RegionMetadata::new(id, start, len, reserved)),
            dirty_ranges: Mutex::new(Vec::new()),
            sparse_flush: AtomicBool::new(false),
        }))
    }

    pub(crate) fn from(db: &Database, index: usize, meta: RegionMetadata) -> Self {
        Self(Arc::new(RegionInner {
            db: db.weak_clone(),
            index,
            meta: RwLock::new(meta),
            dirty_ranges: Mutex::new(Vec::new()),
            sparse_flush: AtomicBool::new(false),
        }))
    }

    #[inline]
    pub fn create_reader(&self) -> Reader {
        Reader::new(self)
    }

    /// Runs `f` with the region's current bytes while holding the mmap read lock.
    ///
    /// Unlike [`Reader`], this is intended for a single scoped read and does not
    /// clone the region or extend a lock guard's lifetime.
    #[inline]
    pub fn with_read_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R {
        let db = self.db();
        let meta = self.meta();
        let start = meta.start();
        let end = start + meta.len();
        drop(meta);

        let mmap = db.mmap();
        f(&mmap[start..end])
    }

    pub fn open_db_read_only_file(&self) -> Result<File> {
        self.db().open_read_only_file()
    }

    /// Ensures the region has at least `capacity` bytes of reserved space.
    ///
    /// Reserving capacity does not change the region's logical length or write
    /// into the added space. When the region cannot grow in place, its current
    /// contents are moved once to a sufficiently large contiguous range.
    pub fn reserve_capacity(&self, capacity: usize) -> Result<()> {
        let capacity = capacity
            .max(PAGE_SIZE)
            .checked_add(PAGE_SIZE_MINUS_1)
            .ok_or(Error::RegionSizeOverflow {
                current: self.meta().reserved(),
                requested: capacity,
            })?
            & !PAGE_SIZE_MINUS_1;

        let db = self.db();
        let index = self.index();
        let meta = self.meta();
        let start = meta.start();
        let len = meta.len();
        let reserved = meta.reserved();
        drop(meta);

        if capacity <= reserved {
            return Ok(());
        }

        let added_reserve = capacity - reserved;
        let mut layout = db.layout_mut();

        // Extend the final region without moving it.
        if layout.is_last_anything(self) {
            {
                let mut meta = self.meta_mut();
                meta.set_reserved(capacity);
            }
            drop(layout);

            if let Err(error) = db.set_min_len(start + capacity) {
                self.meta_mut().set_reserved(reserved);
                return Err(error);
            }

            let regions = db.regions();
            self.meta_mut().write_if_dirty(index, &regions);
            return Ok(());
        }

        // Consume a hole immediately following the region.
        let hole_start = start + reserved;
        if layout
            .get_hole(hole_start)
            .is_some_and(|gap| gap >= added_reserve)
        {
            layout.remove_or_compress_hole(hole_start, added_reserve)?;
            self.meta_mut().set_reserved(capacity);
            drop(layout);

            let regions = db.regions();
            self.meta_mut().write_if_dirty(index, &regions);
            return Ok(());
        }

        // Relocate to an existing hole or append a new sparse range.
        let new_start = if let Some(hole_start) = layout.find_smallest_adequate_hole(capacity) {
            layout.remove_or_compress_hole(hole_start, capacity)?;
            layout.reserve(hole_start, capacity);
            drop(layout);
            hole_start
        } else {
            let new_start = layout.len();
            layout.reserve(new_start, capacity);
            drop(layout);

            if let Err(error) = db.set_min_len(new_start + capacity) {
                db.layout_mut().take_reserved(new_start);
                return Err(error);
            }
            new_start
        };

        db.copy(start, new_start, len)?;

        let mut layout = db.layout_mut();
        layout.move_region(new_start, self)?;
        assert_eq!(layout.take_reserved(new_start), Some(capacity));

        if len > 0 {
            self.mark_dirty(0, len);
        }
        let regions = db.regions();
        let mut meta = self.meta_mut();
        meta.set_start(new_start);
        meta.set_reserved(capacity);
        meta.write_if_dirty(index, &regions);

        Ok(())
    }

    /// Appends data to the region. Not durable until `flush()`.
    #[inline]
    pub fn write(&self, data: &[u8]) -> Result<()> {
        self.write_with(data, None, false, false)
    }

    /// Writes data at offset within the region. Not durable until `flush()`.
    #[inline]
    pub fn write_at(&self, data: &[u8], at: usize) -> Result<()> {
        self.write_with(data, Some(at), false, false)
    }

    /// Writes data at an arbitrary reserved offset, growing the logical region
    /// length when needed. Bytes skipped between the old length and `at` must
    /// not be read unless they have been initialized separately.
    ///
    /// This is intended for sparse, fixed-layout storage whose independent
    /// sections are filled out of order.
    #[doc(hidden)]
    #[inline]
    pub fn write_at_grow(&self, data: &[u8], at: usize) -> Result<()> {
        self.write_with(data, Some(at), false, true)
    }

    /// Prevents database-wide flush coalescing from spanning this region's
    /// intentionally sparse internal layout.
    #[doc(hidden)]
    pub fn use_sparse_flush(&self) {
        self.0.sparse_flush.store(true, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn uses_sparse_flush(&self) -> bool {
        self.0.sparse_flush.load(Ordering::Relaxed)
    }

    /// Deallocates initialized but unused bytes inside this region without
    /// changing its logical length.
    #[doc(hidden)]
    pub fn punch_hole(&self, offset: usize, len: usize) -> Result<()> {
        if len == 0 {
            return Ok(());
        }
        let end = offset.checked_add(len).ok_or(Error::RegionSizeOverflow {
            current: offset,
            requested: len,
        })?;
        let meta = self.meta_mut();
        if end > meta.reserved() {
            return Err(Error::WriteOutOfBounds {
                position: end,
                region_len: meta.reserved(),
            });
        }
        let start = meta.start() + offset;
        let db = self.db();
        HolePunch::punch(&db.file(), start, len)
    }

    /// Writes ascending (offset, value) pairs directly to the mmap within region bounds.
    #[inline]
    pub fn batch_write_ordered<T, F>(
        &self,
        mut iter: impl Iterator<Item = (usize, T)>,
        value_len: usize,
        mut write_fn: F,
    ) where
        F: FnMut(&T, &mut [u8]),
    {
        let Some((first_offset, first_value)) = iter.next() else {
            return;
        };

        let meta = self.meta();
        let region_start = meta.start();
        let region_len = meta.len();
        drop(meta);

        let db = self.db();
        let mmap = db.mmap();
        let ptr = mmap.as_ptr() as *mut u8;

        let first_end = first_offset
            .checked_add(value_len)
            .expect("offset + value_len overflow");
        assert!(first_end <= region_len);
        let first_abs_offset = region_start + first_offset;
        let first_slice =
            unsafe { std::slice::from_raw_parts_mut(ptr.add(first_abs_offset), value_len) };
        write_fn(&first_value, first_slice);

        let mut previous_offset = first_offset;
        let mut dirty_end = first_end;
        for (offset, value) in iter {
            debug_assert!(offset >= previous_offset, "batch offsets must be ordered");
            let end_offset = offset
                .checked_add(value_len)
                .expect("offset + value_len overflow");
            assert!(end_offset <= region_len);

            let abs_offset = region_start + offset;
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr.add(abs_offset), value_len) };
            write_fn(&value, slice);
            previous_offset = offset;
            dirty_end = end_offset;
        }

        self.mark_dirty(first_offset, dirty_end - first_offset);
    }

    pub fn truncate(&self, from: usize) -> Result<()> {
        let len = self.meta().len();
        if from == len {
            return Ok(());
        } else if from > len {
            return Err(Error::TruncateInvalid {
                from,
                current_len: len,
            });
        }

        let db = self.db();
        // Lock order: regions -> metadata (top-to-bottom)
        let regions = db.regions();
        let mut meta = self.meta_mut();
        meta.set_len(from);
        meta.write_if_dirty(self.index(), &regions);
        Ok(())
    }

    /// Truncates to `at`, then writes data there.
    #[inline]
    pub fn truncate_write(&self, at: usize, data: &[u8]) -> Result<()> {
        self.write_with(data, Some(at), true, false)
    }

    #[inline]
    fn write_with(
        &self,
        data: &[u8],
        at: Option<usize>,
        truncate: bool,
        allow_grow: bool,
    ) -> Result<()> {
        let db = self.db();
        let index = self.index();
        let meta = self.meta();
        let start = meta.start();
        let reserved = meta.reserved();
        let len = meta.len();
        drop(meta);

        let data_len = data.len();

        if !allow_grow
            && let Some(at_val) = at
            && at_val > len
        {
            return Err(Error::WriteOutOfBounds {
                position: at_val,
                region_len: len,
            });
        }

        let write_offset = at.unwrap_or(len);
        let new_len = at.map_or(len + data_len, |at| {
            let new_len = at + data_len;
            if truncate { new_len } else { new_len.max(len) }
        });
        let write_start = start + write_offset;

        // --- Fits in reserved space ---
        if new_len <= reserved {
            db.write(write_start, data);
            self.mark_dirty_abs(start, write_start, data_len);

            if new_len != len {
                let regions = db.regions();
                let mut meta = self.meta_mut();
                meta.set_len(new_len);
                meta.write_if_dirty(index, &regions);
            }

            return Ok(());
        }

        if reserved == 0 {
            return Err(Error::InvariantViolation(format!(
                "reserved is 0 which would cause infinite loop! start={start}, len={len}, index={index}, new_len={new_len}"
            )));
        }

        let mut new_reserved = reserved;
        while new_len > new_reserved {
            new_reserved = new_reserved
                .checked_mul(2)
                .ok_or(Error::RegionSizeOverflow {
                    current: new_reserved,
                    requested: new_len,
                })?;
        }
        let added_reserve = new_reserved - reserved;

        let copy_len = if truncate { write_offset } else { len };

        trace!(
            "{}: '{}' write_with acquiring layout_mut (need to grow)",
            db,
            self.meta().id()
        );
        let mut layout = db.layout_mut();

        // --- Extend last region in file ---
        if layout.is_last_anything(self) {
            let target_len = start + new_reserved;
            // Update reserved before dropping layout so Layout::len() is correct.
            {
                let mut meta = self.meta_mut();
                meta.set_reserved(new_reserved);
            }
            // Drop layout before set_min_len (needs mmap_mut — would deadlock).
            drop(layout);

            if let Err(e) = db.set_min_len(target_len) {
                let mut meta = self.meta_mut();
                meta.set_reserved(reserved);
                return Err(e);
            }

            db.write(write_start, data);

            self.mark_dirty_abs(start, write_start, data_len);
            let regions = db.regions();
            let mut meta = self.meta_mut();
            meta.set_len(new_len);
            meta.write_if_dirty(index, &regions);

            return Ok(());
        }

        // --- Expand into adjacent hole ---
        let hole_start = start + reserved;
        if layout
            .get_hole(hole_start)
            .is_some_and(|gap| gap >= added_reserve)
        {
            layout.remove_or_compress_hole(hole_start, added_reserve)?;
            let mut meta = self.meta_mut();
            meta.set_reserved(new_reserved);
            drop(meta);
            drop(layout);

            db.write(write_start, data);

            self.mark_dirty_abs(start, write_start, data_len);
            let regions = db.regions();
            let mut meta = self.meta_mut();
            meta.set_len(new_len);
            meta.write_if_dirty(index, &regions);

            return Ok(());
        }

        // --- Relocate to a hole or append at end ---
        let new_start = if let Some(hole_start) = layout.find_smallest_adequate_hole(new_reserved) {
            debug!(
                "{}: '{}' relocating to hole at {} (need {})",
                db,
                self.meta().id(),
                hole_start,
                new_reserved
            );
            layout.remove_or_compress_hole(hole_start, new_reserved)?;
            layout.reserve(hole_start, new_reserved);
            drop(layout);
            hole_start
        } else {
            let new_start = layout.len();
            let target_len = new_start + new_reserved;
            debug!(
                "{}: '{}' allocating at end {} (need {})",
                db,
                self.meta().id(),
                new_start,
                new_reserved
            );
            // Reserve before dropping layout so other threads see updated len().
            layout.reserve(new_start, new_reserved);
            // Drop layout before set_min_len (needs mmap_mut — would deadlock).
            drop(layout);

            if let Err(e) = db.set_min_len(target_len) {
                let mut layout = db.layout_mut();
                layout.take_reserved(new_start);
                return Err(e);
            }
            new_start
        };

        db.copy(start, new_start, copy_len)?;
        db.write(new_start + write_offset, data);

        trace!(
            "{}: '{}' write_with re-acquiring layout_mut (after relocation)",
            db,
            self.meta().id()
        );
        let mut layout = db.layout_mut();
        layout.move_region(new_start, self)?;
        assert!(layout.take_reserved(new_start) == Some(new_reserved));

        self.mark_dirty(0, new_len);
        let regions = db.regions();
        let mut meta = self.meta_mut();
        meta.set_start(new_start);
        meta.set_reserved(new_reserved);
        meta.set_len(new_len);
        meta.write_if_dirty(index, &regions);

        Ok(())
    }

    pub fn rename(&self, new_id: &str) -> Result<()> {
        let old_id = self.meta().id().to_string();
        let db = self.db();
        debug!("{}: rename '{}' -> '{}'", db, old_id, new_id);
        trace!(
            "{}: rename '{}' -> '{}' acquiring regions_mut",
            db, old_id, new_id
        );
        let mut regions = db.regions_mut();
        let mut meta = self.meta_mut();
        let index = self.index();
        regions.rename(&old_id, new_id)?;
        meta.set_id(new_id.to_string());
        meta.write_if_dirty(index, &regions);
        Ok(())
    }

    /// Space becomes reusable after the next `flush()`.
    pub fn remove(self) -> Result<()> {
        let db = self.db();
        let id = self.meta().id().to_string();
        debug!("{}: '{}' remove", db, id);
        trace!("{}: '{}' remove acquiring layout_mut", db, id);
        // Lock order: layout → regions
        let mut layout = db.layout_mut();
        trace!("{}: '{}' remove acquiring regions_mut", db, id);
        let mut regions = db.regions_mut();
        trace!("{}: '{}' remove got locks", db, id);
        layout.remove_region(&self)?;
        regions.remove(&self)?;
        Ok(())
    }

    /// Flushes this region's dirty data and all pending metadata.
    /// Returns whether anything was flushed.
    pub fn flush(&self) -> Result<bool> {
        let db = self.db();
        let dirty_ranges = self.take_dirty_ranges();

        let data_flushed = if !dirty_ranges.is_empty() {
            let region_start = self.meta().start();
            let mmap = db.mmap();
            for &(start, end) in &dirty_ranges {
                if let Err(error) = mmap.flush_async_range(region_start + start, end - start) {
                    drop(mmap);
                    self.restore_dirty_ranges(&dirty_ranges);
                    return Err(error.into());
                }
            }
            true
        } else {
            false
        };

        // Data MUST be durable before metadata — if we crash after metadata sync
        // but before data sync, metadata could reference unwritten data.
        if data_flushed && let Err(error) = db.file().sync_data() {
            self.restore_dirty_ranges(&dirty_ranges);
            return Err(error.into());
        }

        let metadata_flushed = db.regions().flush()?;

        Ok(data_flushed || metadata_flushed)
    }

    #[inline(always)]
    pub fn ptr_eq(&self, other: &Region) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    #[inline(always)]
    pub(crate) fn arc(&self) -> &Arc<RegionInner> {
        &self.0
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        self.0.index
    }

    #[inline(always)]
    pub fn meta(&self) -> RwLockReadGuard<'_, RegionMetadata> {
        self.0.meta.read()
    }

    #[inline(always)]
    pub(crate) fn meta_mut(&self) -> RwLockWriteGuard<'_, RegionMetadata> {
        self.0.meta.write()
    }

    #[inline(always)]
    pub fn db(&self) -> Database {
        self.0.db.upgrade()
    }

    #[inline]
    pub fn mark_dirty(&self, offset: usize, len: usize) {
        if len == 0 {
            return;
        }
        let end = offset + len;
        let mut ranges = self.0.dirty_ranges.lock();
        let mut start = offset;
        let mut end = end;
        let at = ranges.partition_point(|&(_, range_end)| range_end < start);
        while at < ranges.len() && ranges[at].0 <= end {
            let range = ranges.remove(at);
            start = start.min(range.0);
            end = end.max(range.1);
        }
        ranges.insert(at, (start, end));
    }

    #[inline]
    fn mark_dirty_abs(&self, region_start: usize, abs_start: usize, len: usize) {
        let offset = abs_start - region_start;
        self.mark_dirty(offset, len);
    }

    #[inline]
    pub(crate) fn take_dirty_ranges(&self) -> Vec<(usize, usize)> {
        mem::take(&mut *self.0.dirty_ranges.lock())
    }

    #[inline]
    pub(crate) fn restore_dirty_ranges(&self, ranges: &[(usize, usize)]) {
        for &(start, end) in ranges {
            self.mark_dirty(start, end - start);
        }
    }
}
