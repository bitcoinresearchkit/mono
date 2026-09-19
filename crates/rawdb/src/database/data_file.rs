use std::{
    fs::File,
    path::Path,
    ptr,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use memmap2::MmapRaw;
use parking_lot::{RwLock, RwLockReadGuard};

use crate::{Error, PAGE_SIZE, Result, dirty_ranges::DirtyRanges};

use super::{hole_punch::punch_hole, locked_file::open_locked_file, mmap::write_to_mmap};

/// Owns the data file, its mapping, and file-size durability.
pub(crate) struct DataFile {
    file: File,
    mmap: RwLock<MmapRaw>,
    len: AtomicUsize,
    dirty: AtomicBool,
}

impl DataFile {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let file = open_locked_file(path)?;
        let len = usize::try_from(file.metadata()?.len()).map_err(|_| Error::FileSizeOverflow {
            requested: usize::MAX,
        })?;
        let mmap = MmapRaw::map_raw(&file)?;
        Ok(Self {
            file,
            mmap: RwLock::new(mmap),
            len: AtomicUsize::new(len),
            dirty: AtomicBool::new(false),
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    pub(crate) fn read(&self) -> RwLockReadGuard<'_, MmapRaw> {
        self.mmap.read_recursive()
    }

    pub(crate) fn ensure_len(&self, len: usize) -> Result<()> {
        let len = Self::aligned_len(len)?;
        if self.len() >= len {
            return Ok(());
        }
        let mut mmap = self.mmap.write();
        let current_len = self.len();
        if current_len >= len {
            return Ok(());
        }
        let target_len =
            Self::aligned_len(len.max(current_len.saturating_mul(2)).max(1024 * 1024))?;
        self.file.set_len(target_len as u64)?;
        self.dirty.store(true, Ordering::Relaxed);
        *mmap = MmapRaw::map_raw(&self.file)?;
        // A failed remap leaves the old capacity usable; a later growth retries.
        self.len.store(target_len, Ordering::Relaxed);
        Ok(())
    }

    fn aligned_len(len: usize) -> Result<usize> {
        len.checked_next_multiple_of(PAGE_SIZE)
            .filter(|&len| len <= isize::MAX as usize)
            .ok_or(Error::FileSizeOverflow { requested: len })
    }

    /// # Safety
    /// The caller must hold exclusive access to the destination region.
    pub(crate) unsafe fn write(&self, start: usize, bytes: &[u8]) {
        unsafe { write_to_mmap(&self.read(), start, bytes) };
    }

    /// # Safety
    /// The caller exclusively owns the source region and the free destination.
    pub(crate) unsafe fn copy(&self, src: usize, dst: usize, len: usize) {
        let mmap = self.read();
        assert!(src <= mmap.len() && len <= mmap.len() - src);
        assert!(dst <= mmap.len() && len <= mmap.len() - dst);
        assert!(src + len <= dst || dst + len <= src);
        // No shared reference over the entire mapping is formed.
        unsafe {
            ptr::copy_nonoverlapping(mmap.as_ptr().add(src), mmap.as_mut_ptr().add(dst), len)
        };
    }

    pub(crate) fn flush(&self, ranges: &DirtyRanges, metadata_dirty: bool) -> Result<()> {
        if !ranges.is_empty() {
            let mmap = self.read();
            for range in ranges.iter() {
                mmap.flush_async_range(range.start, range.len())?;
            }
        }
        // Metadata can refer to newly grown space even without dirty data.
        self.sync(metadata_dirty || !ranges.is_empty())
    }

    pub(crate) fn sync(&self, force: bool) -> Result<()> {
        let dirty = self.dirty.swap(false, Ordering::Relaxed);
        if (dirty || force)
            && let Err(error) = self.file.sync_all()
        {
            self.dirty.store(true, Ordering::Relaxed);
            return Err(error.into());
        }
        Ok(())
    }

    pub(crate) fn punch_hole(&self, start: usize, len: usize) -> Result<()> {
        self.dirty.store(true, Ordering::Relaxed);
        punch_hole(&self.file, start, len)
    }
}
