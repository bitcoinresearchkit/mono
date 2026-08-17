use memmap2::MmapMut;
use parking_lot::RwLockReadGuard;

use crate::{Database, Region};

/// Zero-copy reader with a snapshot of region start/len.
///
/// Drop as soon as possible — blocks mmap remapping while held.
#[must_use = "Reader holds locks and should be used for reading"]
pub struct Reader {
    // SAFETY: Drop order matters. `mmap` (the lock guard) must drop before `_db`
    // (the Arc). Rust drops fields in declaration order, so this is correct.
    mmap: RwLockReadGuard<'static, MmapMut>,
    start: usize,
    len: usize,
    _region: Region,
    _db: Database,
}

impl Reader {
    #[inline]
    pub(crate) fn new(region: &Region) -> Self {
        let db = region.db();
        let region = region.clone();

        let meta = region.meta();
        let start = meta.start();
        let len = meta.len();
        drop(meta);

        // SAFETY: Transmute extends the guard lifetime to 'static. This is safe
        // because `_db` (the Arc) outlives `mmap` (the guard) — see struct field order.
        let mmap: RwLockReadGuard<'static, MmapMut> = unsafe { std::mem::transmute(db.mmap()) };

        Self {
            _db: db,
            _region: region,
            start,
            len,
            mmap,
        }
    }

    /// Reads without checking the region boundary. The mmap boundary remains checked.
    #[inline(always)]
    pub fn unchecked_read(&self, offset: usize, len: usize) -> &[u8] {
        let start = self.start() + offset;
        let end = start + len;
        &self.mmap[start..end]
    }

    #[inline(always)]
    pub fn read(&self, offset: usize, len: usize) -> &[u8] {
        assert!(offset + len <= self.len());
        self.unchecked_read(offset, len)
    }

    #[inline(always)]
    fn start(&self) -> usize {
        self.start
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn read_all(&self) -> &[u8] {
        self.read(0, self.len())
    }

    /// Slice from offset to end of mmap (may extend past region boundary).
    #[inline(always)]
    pub fn prefixed(&self, offset: usize) -> &[u8] {
        assert!(offset <= self.len());
        let start = self.start() + offset;
        &self.mmap[start..]
    }
}
