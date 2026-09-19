mod dirty_write;
mod inner;
pub(crate) mod metadata;
mod reader;
pub(crate) mod residency;

pub use metadata::RegionMetadata;
pub use reader::Reader;

use std::{
    fs::File,
    slice,
    sync::{Arc, atomic::Ordering},
};

use parking_lot::RwLockReadGuard;

use crate::{Database, Error, PAGE_SIZE, Result};

use self::{
    dirty_write::DirtyWrite,
    inner::RegionInner,
    metadata::MAX_RESERVED_SIZE,
    residency::{MMAP_RESIDENCY_MIN_BYTES, is_range_resident},
};

/// Named, dynamically-sized region within a database.
#[derive(Debug, Clone)]
#[must_use = "Region should be stored to access the data"]
pub struct Region(pub(crate) Arc<RegionInner>);

impl Region {
    pub(crate) fn new(db: &Database, index: usize, meta: RegionMetadata) -> Self {
        Self(Arc::new(RegionInner::new(db, index, meta)))
    }

    pub fn create_reader(&self) -> Reader {
        Reader::new(self)
    }

    /// Holds the region stable for a scoped read, including buffered file I/O.
    /// Drop the guard before mutating this region.
    pub fn read_lock(&self) -> RwLockReadGuard<'_, ()> {
        self.0.access.read_recursive()
    }

    /// Runs `f` with the region's bytes. The callback must not mutate this region
    /// or grow the database mapping while the borrowed bytes are alive.
    #[inline]
    pub fn with_read_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R {
        let _access = self.read_lock();
        let db = self.db();
        let meta = self.meta();
        let start = meta.start();
        let len = meta.len();
        drop(meta);
        let mmap = db.inner.data.read();
        assert!(start <= mmap.len() && len <= mmap.len() - start);
        // SAFETY: access prevents changes to this region, and mmap prevents
        // remapping. Only this region's initialized bytes are borrowed.
        f(unsafe { slice::from_raw_parts(mmap.as_ptr().add(start), len) })
    }

    pub fn open_db_read_only_file(&self) -> Result<File> {
        self.db().open_read_only_file()
    }

    /// Returns whether mmap is preferred for a sequential region-relative byte range.
    ///
    /// Small ranges use mmap directly. Larger ranges use mmap only when conservative
    /// sampling finds the first and last pages plus one page per 16 MiB resident.
    /// Any missing sample or probing error prefers buffered I/O.
    /// This is a performance hint, not range validation: short ranges skip bounds
    /// checks. The caller must still validate reads against the region length.
    pub fn prefers_mmap(&self, offset: usize, len: usize) -> bool {
        if len < MMAP_RESIDENCY_MIN_BYTES {
            return true;
        }

        let _access = self.read_lock();
        let meta = self.meta();
        let Some(end) = offset.checked_add(len) else {
            return false;
        };
        if end > meta.len() {
            return false;
        }
        let Some(absolute_start) = meta.start().checked_add(offset) else {
            return false;
        };
        drop(meta);

        let db = self.db();
        let mmap = db.inner.data.read();
        is_range_resident(&mmap, absolute_start, len)
    }

    /// Ensures the region has at least `capacity` bytes of reserved space.
    ///
    /// Reserving capacity does not change the region's logical length or write
    /// into the added space. When the region cannot grow in place, its current
    /// contents are moved once to a sufficiently large contiguous range.
    pub fn reserve_capacity(&self, capacity: usize) -> Result<()> {
        let db = self.db();
        let _access = self.0.access.write();
        let capacity = capacity
            .max(PAGE_SIZE)
            .checked_next_multiple_of(PAGE_SIZE)
            .filter(|&size| size <= MAX_RESERVED_SIZE)
            .ok_or_else(|| Error::RegionSizeOverflow {
                current: self.meta().reserved(),
                requested: capacity,
            })?;
        self.reserve_inner(&db, capacity)
    }

    /// Caller holds this region's access lock. Capacity changes preserve all
    /// existing logical bytes, so an intervening flush can persist them safely.
    fn reserve_inner(&self, db: &Database, capacity: usize) -> Result<()> {
        let meta = self.meta();
        let start = meta.start();
        let len = meta.len();
        let reserved = meta.reserved();
        drop(meta);
        if capacity <= reserved {
            return Ok(());
        }

        loop {
            let writes = db.inner.writes.read();
            let mut layout = db.layout_mut();
            let added = capacity - reserved;
            let next = start + reserved;
            let adjacent = layout.get_hole(next).is_some_and(|hole| hole >= added);
            let layout_end = layout.end();
            let in_place = next == layout_end || adjacent;
            let hole = if in_place {
                None
            } else {
                layout.find_smallest_adequate_hole(capacity)
            };
            let new_start = if in_place {
                start
            } else {
                hole.unwrap_or(layout_end)
            };
            let end = new_start
                .checked_add(capacity)
                .ok_or(Error::FileSizeOverflow {
                    requested: capacity,
                })?;
            if end > db.file_len() {
                // Never wait for mmap readers while holding allocation or
                // mutation-barrier locks. No tentative allocation needs rollback.
                drop(layout);
                drop(writes);
                db.inner.data.ensure_len(end)?;
                continue;
            }
            if adjacent {
                layout.consume_hole(next, added);
            } else if !in_place {
                // SAFETY: access owns the source region; layout owns the free destination.
                unsafe { db.inner.data.copy(start, new_start, len) };
                if hole.is_some() {
                    layout.consume_hole(new_start, capacity);
                }
                layout.move_region(new_start, self);
                self.0.mark_dirty(0, len);
            }
            let regions = db.regions();
            let mut meta = self.0.meta.write();
            meta.set_start(new_start);
            meta.set_reserved(capacity);
            self.0.tail_needs_punch.store(true, Ordering::Relaxed);
            regions.update_bounds(self.index(), &meta);
            return Ok(());
        }
    }

    /// Appends data to the region. Not durable until `flush()`.
    #[inline]
    pub fn write(&self, data: &[u8]) -> Result<()> {
        self.write_with(data, None, false)
    }

    /// Writes at or before the current end, extending the length if needed.
    /// Not durable until `flush()`.
    #[inline]
    pub fn write_at(&self, data: &[u8], at: usize) -> Result<()> {
        self.write_with(data, Some(at), false)
    }

    /// Writes ascending (offset, value) pairs within the region's current length.
    /// Panics for unordered or out-of-bounds offsets. Completed writes remain
    /// dirty if the iterator or callback panics. The callback must not reenter
    /// this region or resize/flush the database.
    #[inline]
    pub fn batch_write_ordered<T, F>(
        &self,
        mut iter: impl Iterator<Item = (usize, T)>,
        value_len: usize,
        mut write_fn: F,
    ) where
        F: FnMut(&T, &mut [u8]),
    {
        let Some(first) = iter.next() else {
            return;
        };
        let db = self.db();
        let _access = self.0.access.write();
        let _writes = db.inner.writes.read();
        let meta = self.meta();
        let region_start = meta.start();
        let region_len = meta.len();
        drop(meta);
        let mmap = db.inner.data.read();
        assert!(region_start <= mmap.len() && region_len <= mmap.len() - region_start);
        // SAFETY: the access lock excludes all other users of this region's
        // bytes, and the mmap guard prevents remapping for the entire batch.
        let bytes =
            unsafe { slice::from_raw_parts_mut(mmap.as_mut_ptr().add(region_start), region_len) };
        let mut dirty = DirtyWrite {
            region: &self.0,
            start: first.0,
            end: first.0,
        };
        let mut previous = first.0;
        let mut write = |offset, value: &T| {
            assert!(offset >= previous, "batch offsets must be ordered");
            assert!(offset <= region_len && value_len <= region_len - offset);
            dirty.end = offset + value_len;
            previous = offset;
            write_fn(value, &mut bytes[offset..dirty.end]);
        };
        // Chaining the first item back into the iterator slows large batches.
        write(first.0, &first.1);
        for (offset, value) in iter {
            write(offset, &value);
        }
    }

    /// Keeps the first `from` bytes without changing reserved capacity.
    /// Returns an error if `from` exceeds the current length.
    pub fn truncate(&self, from: usize) -> Result<()> {
        let db = self.db();
        let _access = self.0.access.write();
        let _writes = db.inner.writes.read();
        let regions = db.regions();
        let mut meta = self.0.meta.write();
        if from > meta.len() {
            return Err(Error::TruncateInvalid {
                from,
                current_len: meta.len(),
            });
        }
        if from != meta.len() {
            meta.set_len(from);
            self.0.tail_needs_punch.store(true, Ordering::Relaxed);
            regions.update_bounds(self.index(), &meta);
        }
        Ok(())
    }

    /// Truncates to `at`, then writes data there.
    #[inline]
    pub fn truncate_write(&self, at: usize, data: &[u8]) -> Result<()> {
        self.write_with(data, Some(at), true)
    }

    #[inline]
    fn write_with(&self, data: &[u8], at: Option<usize>, truncate: bool) -> Result<()> {
        let db = self.db();
        let _access = self.0.access.write();
        let meta = self.meta();
        // Access keeps this stable unless this write relocates the region.
        let mut start = meta.start();
        let len = meta.len();
        let reserved = meta.reserved();
        drop(meta);
        let offset = at.unwrap_or(len);
        if offset > len {
            return Err(Error::WriteOutOfBounds {
                position: offset,
                region_len: len,
            });
        }
        let end = offset
            .checked_add(data.len())
            .filter(|&end| end <= MAX_RESERVED_SIZE)
            .ok_or(Error::RegionSizeOverflow {
                current: reserved,
                requested: data.len(),
            })?;
        let new_len = if truncate { end } else { end.max(len) };
        if new_len > reserved {
            let mut capacity = reserved;
            while capacity < new_len {
                capacity = capacity.saturating_mul(2).min(MAX_RESERVED_SIZE);
            }
            self.reserve_inner(&db, capacity)?;
            start = self.meta().start();
        }

        let _writes = db.inner.writes.read();
        // SAFETY: access excludes readers and overlapping writes to this region.
        unsafe { db.inner.data.write(start + offset, data) };
        self.0.mark_dirty(offset, data.len());
        if new_len != len {
            let regions = db.regions();
            let mut meta = self.0.meta.write();
            meta.set_len(new_len);
            if new_len < len {
                self.0.tail_needs_punch.store(true, Ordering::Relaxed);
            }
            regions.update_bounds(self.index(), &meta);
        }
        Ok(())
    }

    /// Space becomes reusable after the next `flush()`. On failure the region
    /// and its allocation are unchanged.
    pub fn remove(self) -> Result<()> {
        let db = self.db();
        let _writes = db.inner.writes.read();
        let mut layout = db.layout_mut();
        let mut regions = db.regions_mut();
        regions.check_removable(&self)?;
        layout.remove_region(&self);
        regions.remove(&self);
        Ok(())
    }

    /// Flushes all dirty data and metadata in this region's database.
    /// The metadata file is shared, so all regions' data must precede its sync.
    /// Returns whether any region data or region metadata was flushed.
    pub fn flush(&self) -> Result<bool> {
        let db = self.db();
        let _writes = db.inner.writes.write();
        db.flush_inner()
            .map(|(regions, metadata)| regions > 0 || metadata)
    }

    pub(crate) fn ptr_eq(&self, other: &Region) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn index(&self) -> usize {
        self.0.index
    }

    /// Borrows the region's current metadata. Drop this guard before mutations.
    /// Hold [`Self::read_lock`] as well when using its bounds for buffered I/O.
    pub fn meta(&self) -> RwLockReadGuard<'_, RegionMetadata> {
        self.0.meta.read()
    }

    pub fn db(&self) -> Database {
        self.0.db.upgrade()
    }
}
