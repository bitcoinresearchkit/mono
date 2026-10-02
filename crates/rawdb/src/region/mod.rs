mod dirty_write;
mod inner;
pub(crate) mod metadata;
mod reader;
pub(crate) mod residency;
#[cfg(test)]
mod tests;

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
    #[inline]
    pub fn read_lock(&self) -> RwLockReadGuard<'_, ()> {
        self.0.access.read_recursive()
    }

    /// Runs `f` with the region's bytes. The callback must not mutate this region
    /// or grow the database mapping while the borrowed bytes are alive.
    #[inline(always)]
    pub fn with_read_bytes<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R {
        let db = self.db();
        let data = &db.inner.data;
        let _access = self.read_lock();
        // SAFETY: access keeps all metadata fields stable.
        let (start, len, _) = unsafe { self.0.bounds() };
        let mmap = data.read();
        debug_assert!(start <= mmap.len() && len <= mmap.len() - start);
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

        let mmap = db.inner.data.read();
        is_range_resident(&mmap, absolute_start, len)
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
                db.inner.data.ensure_len(end, &db.inner.writes)?;
                continue;
            }
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
            self.0.tail_needs_punch.store(true, Ordering::Relaxed);
            drop(meta);
            drop(layout);
            if !in_place {
                // The layout now reserves the destination. Region access keeps
                // readers out, and the barrier prevents flushing or remapping
                // until the copy and the caller's write have both completed.
                unsafe { db.inner.data.copy(start, new_start, copy_len) };
                unsafe { self.0.mark_dirty(0, copy_len) };
            }
            return Ok(writes);
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

    /// Writes fixed-size values at `at + index * value_len` within the current
    /// region length, in ascending key order. All bounds are checked before writing.
    /// Written bytes remain dirty if a callback or value destructor panics.
    /// Callbacks and destructors may read other regions, but must not read this
    /// region or write, resize, or flush this database.
    #[inline]
    pub fn write_indexed<T>(
        &self,
        values: BTreeMap<usize, T>,
        value_len: usize,
        at: usize,
        mut write_fn: impl FnMut(&T, &mut [u8]),
    ) {
        let Some((&first, _)) = values.first_key_value() else {
            return;
        };
        let last = *values.last_key_value().unwrap().0;
        let start = first
            .checked_mul(value_len)
            .and_then(|n| at.checked_add(n))
            .expect("batch offset overflow");
        let end = last
            .checked_mul(value_len)
            .and_then(|n| at.checked_add(n))
            .and_then(|n| n.checked_add(value_len))
            .expect("batch offset overflow");
        let db = self.db();
        let storage = &**db.inner;
        let _access = self.0.access.write();
        let _writes = storage.writes.read();
        // SAFETY: access keeps bounds stable; the barrier prevents remapping.
        let (region_start, region_len, _) = unsafe { self.0.bounds() };
        assert!(end <= region_len, "batch bounds exceed region");
        let _dirty = DirtyWrite {
            region: &self.0,
            start,
            end,
        };
        // SAFETY: at <= end <= region_len; access excludes readers and other
        // writers, and the barrier keeps the validated mapping in place.
        let ptr = unsafe { storage.data.mapping().as_mut_ptr().add(region_start + at) };
        for (index, value) in values {
            // SAFETY: this owned map has immutable usize keys in [first, last].
            // The checked maximum proves every record fits without overflow.
            // Each exclusive borrow ends before the next callback.
            let bytes = unsafe { slice::from_raw_parts_mut(ptr.add(index * value_len), value_len) };
            write_fn(&value, bytes);
        }
    }

    /// Writes ascending (offset, value) pairs within the region's current length.
    /// The first and last offsets define the batch's bounds. Every write must
    /// fall within those bounds. Both endpoints are read before the middle;
    /// write callbacks still run in forward order.
    /// Written bytes remain dirty if the iterator or callback panics. The callback
    /// may read other regions, but must not write, resize, or flush this database.
    #[inline]
    pub fn batch_write_ordered<T, F>(
        &self,
        mut iter: impl DoubleEndedIterator<Item = (usize, T)>,
        value_len: usize,
        mut write_fn: F,
    ) where
        F: FnMut(&T, &mut [u8]),
    {
        let Some(first) = iter.next() else {
            return;
        };
        let last = iter.next_back();
        let last_offset = last.as_ref().map_or(first.0, |value| value.0);
        let db = self.db();
        let storage = &**db.inner;
        let _access = self.0.access.write();
        let _writes = storage.writes.read();
        // SAFETY: access keeps bounds stable; the barrier prevents remapping.
        let (region_start, region_len, _) = unsafe { self.0.bounds() };
        let end = last_offset
            .checked_add(value_len)
            .expect("batch offset overflow");
        assert!(
            first.0 <= last_offset && end <= region_len,
            "batch bounds exceed region"
        );
        // SAFETY: the barrier prevents remapping, and allocation/import validate bounds.
        let ptr = unsafe {
            storage
                .data
                .mapping()
                .as_mut_ptr()
                .add(region_start + first.0)
        };
        let _dirty = DirtyWrite {
            region: &self.0,
            start: first.0,
            end,
        };
        let span_len = end - first.0;
        // SAFETY: the validated span is exclusively held and cannot be remapped.
        let bytes = unsafe { slice::from_raw_parts_mut(ptr, span_len) };
        let mut write = |offset: usize, value: &T| {
            let relative = offset.wrapping_sub(first.0);
            let dst = bytes
                .get_mut(relative..)
                .and_then(|tail| tail.get_mut(..value_len))
                .expect("offset outside batch bounds");
            write_fn(value, dst);
        };
        write(first.0, &first.1);
        if let Some((offset, value)) = iter.next() {
            write(offset, &value);
            for (offset, value) in iter {
                write(offset, &value);
            }
        }
        if let Some((offset, value)) = last {
            write(offset, &value);
        }
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
        let mut meta = self.0.meta.write();
        if from > meta.byte_len() {
            return Err(Error::TruncateInvalid {
                from,
                current_len: meta.byte_len(),
            });
        }
        if from != meta.byte_len() {
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
        // SAFETY: region access and the mutation barrier remain held.
        unsafe { self.0.mark_dirty(offset, data.len()) };
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
