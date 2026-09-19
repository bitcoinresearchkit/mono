use std::{mem, slice};

use memmap2::MmapRaw;
use parking_lot::RwLockReadGuard;

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
    // Both guards drop before their owning handles below.
    _access: RwLockReadGuard<'static, ()>,
    start: usize,
    len: usize,
    _region: Region,
    _db: Database,
}

impl Reader {
    #[inline]
    pub(super) fn new(region: &Region) -> Self {
        let db = region.db();
        let region = region.clone();

        // SAFETY: `_region` outlives this guard, and its allocation is stable.
        let access: RwLockReadGuard<'static, ()> = unsafe { mem::transmute(region.read_lock()) };

        let meta = region.meta();
        let start = meta.start();
        let len = meta.len();
        drop(meta);

        // SAFETY: Transmute extends the guard lifetime to 'static. This is safe
        // because `_db` (the Arc) outlives `mmap` (the guard) — see struct field order.
        let mmap: RwLockReadGuard<'static, MmapRaw> =
            unsafe { mem::transmute(db.inner.data.read()) };
        assert!(start <= mmap.len() && len <= mmap.len() - start);

        Self {
            mmap,
            _access: access,
            start,
            len,
            _region: region,
            _db: db,
        }
    }

    pub fn read(&self, offset: usize, len: usize) -> &[u8] {
        assert!(len <= self.len() && offset <= self.len() - len);
        // SAFETY: construction checked the complete region against the mapping;
        // the access and mmap guards keep its bytes and address stable.
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
