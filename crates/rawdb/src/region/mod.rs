#[cfg(unix)]
mod indexed;
mod inner;
pub(crate) mod metadata;
mod reader;
pub(crate) mod residency;

pub use metadata::RegionMetadata;
pub use reader::Reader;

use std::{
    collections::BTreeMap,
    fs::File,
    slice,
    sync::{Arc, atomic::Ordering},
};

use parking_lot::RwLockReadGuard;

use crate::{Database, Error, PAGE_SIZE, Result};

use self::{
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
    #[inline]
    pub fn read_lock(&self) -> RwLockReadGuard<'_, ()> {
        self.0.access.read_recursive()
    }

    /// Runs `f` with the region's bytes from `offset` to its end. The callback must not rewrite,
    /// truncate or move this region (or append past its reservation) while the bytes are alive.
    #[inline(always)]
    pub fn with_read_bytes<R>(&self, offset: usize, f: impl FnOnce(&[u8]) -> R) -> R {
        let db = self.db();
        let data = &db.inner.data;
        let _access = self.read_lock();
        // SAFETY: access keeps start and reservation stable; the length only grows under it.
        let (start, len, _) = unsafe { self.0.bounds() };
        assert!(offset <= len);
        let mmap = data.mapping();
        debug_assert!(start <= mmap.len() && len <= mmap.len() - start);
        // SAFETY: access prevents changes to these bytes (appends go past them), and the mapping
        // never moves.
        f(unsafe { slice::from_raw_parts(mmap.as_ptr().add(start + offset), len - offset) })
    }

    /// Runs `f` on the `len` bytes at `offset`, borrowing nothing else: a value read this way
    /// never overlaps [`Self::write_indexed_unread`]'s writes of other values.
    pub fn with_read_range<R>(&self, offset: usize, len: usize, f: impl FnOnce(&[u8]) -> R) -> R {
        let db = self.db();
        let data = &db.inner.data;
        let _access = self.read_lock();
        // SAFETY: access keeps start and reservation stable; the length only grows under it.
        let (start, region_len, _) = unsafe { self.0.bounds() };
        assert!(len <= region_len && offset <= region_len - len);
        let mmap = data.mapping();
        // SAFETY: the range lies within the region, access prevents changes to these bytes (but
        // for unread writes, which never write them), and the mapping never moves.
        f(unsafe { slice::from_raw_parts(mmap.as_ptr().add(start + offset), len) })
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

        let db = self.db();
        let _access = self.read_lock();
        let meta = self.meta();
        let Some(end) = offset.checked_add(len) else {
            return false;
        };
        if end > meta.byte_len() {
            return false;
        }
        let Some(absolute_start) = meta.start().checked_add(offset) else {
            return false;
        };
        drop(meta);

        is_range_resident(db.inner.data.mapping(), absolute_start, len)
    }

    /// Ensures the region has at least `capacity` bytes of reserved space.
    ///
    /// Reserving capacity does not change the region's logical length or write
    /// into the added space. When the region cannot grow in place, its current
    /// contents are moved once to a sufficiently large contiguous range.
    pub fn reserve_capacity(&self, capacity: usize) -> Result<()> {
        if capacity <= self.meta().reserved() {
            return Ok(());
        }
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
        let meta = self.meta();
        if capacity <= meta.reserved() {
            return Ok(());
        }
        let len = meta.byte_len();
        drop(meta);
        let _writes = self.reserve_inner(&db, capacity, len)?;
        db.regions().update_bounds(self.index(), &self.meta());
        Ok(())
    }

    /// Grows the in-memory allocation while the caller holds region access.
    /// Keep the returned barrier until replacement bytes and metadata are written;
    /// a truncating write copies only its prefix. Metadata slots stay unchanged here.
    fn reserve_inner<'a>(
        &self,
        db: &'a Database,
        capacity: usize,
        copy_len: usize,
    ) -> Result<RwLockReadGuard<'a, ()>> {
        let meta = self.meta();
        let start = meta.start();
        let reserved = meta.reserved();
        drop(meta);
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
        // Growth stays inside the mapping: it waits for no one.
        db.inner.data.ensure_len(end)?;
        if adjacent {
            layout.consume_hole(next, added);
        } else if !in_place {
            if hole.is_some() {
                layout.consume_hole(new_start, capacity);
            }
            layout.move_region(new_start, self);
        }
        let mut meta = self.0.meta.write();
        meta.set_start(new_start);
        meta.set_reserved(capacity);
        self.mark_tail_punch(db);
        drop(meta);
        drop(layout);
        if !in_place {
            // The layout now reserves the destination. Region access keeps
            // readers out, and the barrier prevents flushing until the copy
            // and the caller's write have both completed.
            unsafe { db.inner.data.copy(start, new_start, copy_len) };
        }
        Ok(writes)
    }

    /// Writes at or before the current end, extending the length if needed.
    #[inline]
    pub fn write_at(&self, data: &[u8], at: usize) -> Result<()> {
        self.write_with(data, Some(at), false)
    }

    /// Writes fixed-size values at `at + index * value_len` within the current region length, in no
    /// particular order. All bounds are checked before writing. On unix the values go in whole-page
    /// chunks on a few threads (see `indexed`); elsewhere they are stored through the mapping. If a
    /// callback or value destructor panics, what was already written stays written. Callbacks and
    /// destructors must not access this database: this region's access and the database's mutation
    /// barrier are held throughout, so waiting on either from a callback would deadlock.
    pub fn write_indexed<T: Sync>(
        &self,
        values: BTreeMap<usize, T>,
        value_len: usize,
        at: usize,
        write_fn: impl Fn(&T, &mut [u8]) + Sync,
    ) -> Result<()> {
        self.write_indexed_with(values, value_len, at, write_fn, true)
    }

    /// [`Self::write_indexed`] while readers keep reading the region (the rest of the written pages
    /// is rewritten with its own bytes).
    ///
    /// # Safety
    /// Until it returns, nothing reads or borrows the written values: the region's readers borrow
    /// only the values they read ([`Self::with_read_range`]), never the written ones, and hold no
    /// [`Reader`] or wider slice.
    pub unsafe fn write_indexed_unread<T: Sync>(
        &self,
        values: BTreeMap<usize, T>,
        value_len: usize,
        at: usize,
        write_fn: impl Fn(&T, &mut [u8]) + Sync,
    ) -> Result<()> {
        self.write_indexed_with(values, value_len, at, write_fn, false)
    }

    fn write_indexed_with<T: Sync>(
        &self,
        values: BTreeMap<usize, T>,
        value_len: usize,
        at: usize,
        write_fn: impl Fn(&T, &mut [u8]) + Sync,
        exclude_readers: bool,
    ) -> Result<()> {
        let Some((&last, _)) = values.last_key_value() else {
            return Ok(());
        };
        let end = last
            .checked_mul(value_len)
            .and_then(|n| at.checked_add(n))
            .and_then(|n| n.checked_add(value_len))
            .expect("batch offset overflow");
        let db = self.db();
        let storage = &**db.inner;
        // Exclusive access keeps readers and other writers out; shared access with the writer lock,
        // only other writers. Either keeps the region's bounds stable.
        let _exclusive = exclude_readers.then(|| self.0.access.write());
        let _shared =
            (!exclude_readers).then(|| (self.0.access.read_recursive(), self.0.writer.lock()));
        let _writes = storage.writes.read();
        // SAFETY: access keeps bounds stable; the barrier keeps flushing and hole punching out.
        let (region_start, region_len, _reserved) = unsafe { self.0.bounds() };
        assert!(end <= region_len, "batch bounds exceed region");
        let base = region_start + at;
        // SAFETY: at <= end <= region_len, so every value lies within this region; no reader reads
        // these values (access excludes readers, or the caller vouches for them), and the mapping
        // never moves.
        let store = |index: usize, value: &T| {
            let bytes = unsafe {
                let ptr = storage
                    .data
                    .mapping()
                    .as_mut_ptr()
                    .add(base + index * value_len);
                slice::from_raw_parts_mut(ptr, value_len)
            };
            write_fn(value, bytes);
        };
        #[cfg(unix)]
        indexed::write(
            &storage.data,
            region_start,
            _reserved,
            base,
            value_len,
            values,
            &store,
            &write_fn,
        )?;
        #[cfg(not(unix))]
        for (index, value) in &values {
            store(*index, value);
        }
        Ok(())
    }

    /// Keeps the first `from` bytes without changing reserved capacity.
    /// Returns an error if `from` exceeds the current length.
    pub fn truncate(&self, from: usize) -> Result<()> {
        if from == self.meta().byte_len() {
            return Ok(());
        }
        let db = self.db();
        let _access = self.0.access.write();
        let _writes = db.inner.writes.read();
        let regions = db.regions();
        let meta = self.0.meta.read();
        if from > meta.byte_len() {
            return Err(Error::TruncateInvalid {
                from,
                current_len: meta.byte_len(),
            });
        }
        if from != meta.byte_len() {
            meta.set_len(from);
            self.mark_tail_punch(&db);
            regions.update_bounds(self.index(), &meta);
        }
        Ok(())
    }

    /// Truncates to `at`, then writes data there.
    #[inline]
    pub fn truncate_write(&self, at: usize, data: &[u8]) -> Result<()> {
        self.write_with(data, Some(at), true)
    }

    /// An append within the reservation shares region access with readers: it writes past every
    /// reader's bounds, then publishes the length. Anything else, or growth past the reservation,
    /// takes access exclusively.
    #[inline]
    fn write_with(&self, data: &[u8], at: Option<usize>, truncate: bool) -> Result<()> {
        if data.is_empty() {
            let Some(offset) = at else {
                return Ok(());
            };
            let len = self.meta().byte_len();
            if offset > len {
                return Err(Error::WriteOutOfBounds {
                    position: offset,
                    region_len: len,
                });
            }
            if !truncate || offset == len {
                return Ok(());
            }
        }
        let db = self.db();
        {
            let _access = self.0.access.read_recursive();
            let _writer = self.0.writer.lock();
            // SAFETY: shared access keeps start and reservation stable; the writer lock, the length.
            let (start, len, reserved) = unsafe { self.0.bounds() };
            let offset = at.unwrap_or(len);
            let end = offset.saturating_add(data.len());
            let new_len = if truncate { end } else { end.max(len) };
            if offset == len && new_len <= reserved {
                let _writes = db.inner.writes.read();
                // SAFETY: past every reader's bounds: no reference covers these bytes.
                unsafe { db.inner.data.write(start + offset, data) };
                if new_len != len {
                    self.publish_len(&db, new_len, len);
                }
                return Ok(());
            }
        }
        let _access = self.0.access.write();
        // SAFETY: access keeps metadata stable until this operation updates it.
        let (mut start, len, reserved) = unsafe { self.0.bounds() };
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
        let _writes = if new_len > reserved {
            let mut capacity = reserved;
            while capacity < new_len {
                capacity = capacity.saturating_mul(2).min(MAX_RESERVED_SIZE);
            }
            let copy_len = if truncate { offset } else { len };
            let writes = self.reserve_inner(&db, capacity, copy_len)?;
            start = self.meta().start();
            writes
        } else {
            db.inner.writes.read()
        };
        // SAFETY: access excludes readers and overlapping writes to this region.
        unsafe { db.inner.data.write(start + offset, data) };
        if new_len != len {
            self.publish_len(&db, new_len, len);
        }
        Ok(())
    }

    /// Records a length the written bytes now cover, after them.
    fn publish_len(&self, db: &Database, new_len: usize, len: usize) {
        let regions = db.regions();
        let meta = self.0.meta.read();
        meta.set_len(new_len);
        if new_len < len {
            self.mark_tail_punch(db);
        }
        regions.update_bounds(self.index(), &meta);
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
        db.mark_reclaim_pending();
        Ok(())
    }

    /// Marks the space past this region's length for the next compaction.
    pub(crate) fn mark_tail_punch(&self, db: &Database) {
        self.0.tail_needs_punch.store(true, Ordering::Relaxed);
        db.mark_reclaim_pending();
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

    #[inline]
    pub fn db(&self) -> Database {
        self.0.db.upgrade()
    }
}
