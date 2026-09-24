use std::{mem, slice};

use memmap2::MmapRaw;
use parking_lot::{RwLockReadGuard, lock_api::RawRwLock};

use crate::Database;

use super::Region;

/// Zero-copy reader that keeps this region's bytes and bounds stable.
///
/// Drop before modifying this region or growing the database mapping.
#[must_use = "Reader holds locks and should be used for reading"]
pub struct Reader {
    // SAFETY: Drop order matters. `mmap` (the lock guard) must drop before `_db`
    // (the Arc). Rust drops fields in declaration order, so this is correct.
    mmap: RwLockReadGuard<'static, MmapRaw>,
    start: usize,
    len: usize,
    _region: Region,
    _db: Database,
}

impl Reader {
    #[inline]
    pub(super) fn new(region: &Region) -> Self {
        let db = region.db();
        let access = region.read_lock();

        // SAFETY: access keeps metadata stable for the reader's lifetime.
        let (start, len, _) = unsafe { region.0.bounds() };

        // SAFETY: Transmute extends the guard lifetime to 'static. This is safe
        // because `_db` (the Arc) outlives `mmap` (the guard) — see struct field order.
        let mmap: RwLockReadGuard<'static, MmapRaw> =
            unsafe { mem::transmute(db.inner.data.read()) };
        debug_assert!(start <= mmap.len() && len <= mmap.len() - start);

        let reader = Self {
            mmap,
            start,
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
        assert!(len <= self.len() && offset <= self.len() - len);
        // SAFETY: allocation/import validate region bounds;
        // the owned region lock and mmap guard keep bytes and address stable.
        unsafe { slice::from_raw_parts(self.mmap.as_ptr().add(self.start + offset), len) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn read_all(&self) -> &[u8] {
        self.read(0, self.len())
    }

    /// Slice from offset to the end of this region.
    pub fn read_from(&self, offset: usize) -> &[u8] {
        &self.read_all()[offset..]
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        // SAFETY: new transfers exactly one shared lock count after construction
        // succeeds. The region remains owned here, and both locks are released
        // before the database field can join deferred workers.
        unsafe { self._region.0.access.raw().unlock_shared() };
    }
}
