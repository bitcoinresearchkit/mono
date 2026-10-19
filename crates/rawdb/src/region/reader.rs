use std::{mem, slice};

use parking_lot::lock_api::RawRwLock;

use crate::Database;

use super::Region;

/// Zero-copy reader that keeps this region's bytes and bounds stable.
///
/// Drop before modifying this region.
#[must_use = "Reader holds a region read lock"]
pub struct Reader {
    /// The region's first byte in the database mapping, which never moves while `_db` lives.
    base: *const u8,
    len: usize,
    _region: Region,
    _db: Database,
}

impl Reader {
    #[inline]
    pub(super) fn new(region: &Region) -> Self {
        let db = region.db();
        let access = region.read_lock();

        // SAFETY: access keeps start stable for the reader's lifetime; the length only grows.
        let (start, len, _) = unsafe { region.0.bounds() };

        let mapping = db.inner.data.mapping();
        debug_assert!(start <= mapping.len() && len <= mapping.len() - start);
        // SAFETY: start lies within the mapping.
        let base = unsafe { mapping.as_ptr().add(start) };

        let reader = Self {
            base,
            len,
            _region: region.clone(),
            _db: db,
        };
        // The reader already owns the region that locates this lock. Transfer
        // the shared lock count without storing another pointer to that region.
        mem::forget(access);
        reader
    }

    pub fn read(&self, offset: usize, len: usize) -> &[u8] {
        assert!(len <= self.len && offset <= self.len - len);
        // SAFETY: allocation/import validate region bounds; the owned region lock keeps these bytes
        // in place and unchanged (unread writes never run while a reader lives), and the database
        // keeps the mapping alive.
        unsafe { slice::from_raw_parts(self.base.add(offset), len) }
    }

    pub fn read_all(&self) -> &[u8] {
        self.read(0, self.len)
    }

    /// Slice from offset to the end of this region, borrowing nothing before `offset`.
    pub fn read_from(&self, offset: usize) -> &[u8] {
        assert!(offset <= self.len);
        self.read(offset, self.len - offset)
    }
}

// SAFETY: the pointed-to bytes stay in place and unchanged while the reader holds its region's
// shared access (unread writes never run while a reader lives), and its database handle keeps the mapping alive; shared reads need nothing else.
unsafe impl Sync for Reader {}

impl Drop for Reader {
    fn drop(&mut self) {
        // SAFETY: new transfers exactly one shared lock count after construction
        // succeeds. The region remains owned here, and the region lock is released
        // before the database field can join deferred workers.
        unsafe { self._region.0.access.raw().unlock_shared() };
    }
}
