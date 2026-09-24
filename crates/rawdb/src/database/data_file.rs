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

    #[inline]
    pub(crate) fn read(&self) -> RwLockReadGuard<'_, MmapRaw> {
        self.mmap.read_recursive()
    }

    pub(crate) fn ensure_len(&self, len: usize, writes: &RwLock<()>) -> Result<()> {
        let len = Self::aligned_len(len)?;
        if self.len() >= len {
            return Ok(());
        }
        let (mut mmap, _writes) = loop {
            let mmap = self.mmap.write();
            if let Some(writes) = writes.try_write() {
                break (mmap, writes);
            }
            // A batch may hold the barrier while reading another region. Never
            // hold the mapping lock while waiting for that batch to finish.
            drop(mmap);
            drop(writes.write());
        };
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
    /// Hold the database mutation barrier throughout use of this mapping.
    pub(crate) unsafe fn mapping(&self) -> &MmapRaw {
        // Remapping takes both the mapping lock and the exclusive mutation barrier.
        unsafe { &*self.mmap.data_ptr() }
    }

    /// # Safety
    /// Hold the mutation barrier and exclusive access to the destination region.
    pub(crate) unsafe fn write(&self, start: usize, bytes: &[u8]) {
        unsafe { write_to_mmap(self.mapping(), start, bytes) };
    }

    /// # Safety
    /// Hold the mutation barrier and exclusively own the source and destination.
    pub(crate) unsafe fn copy(&self, src: usize, dst: usize, len: usize) {
        let mmap = unsafe { self.mapping() };
        assert!(src <= mmap.len() && len <= mmap.len() - src);
        assert!(dst <= mmap.len() && len <= mmap.len() - dst);
        assert!(src + len <= dst || dst + len <= src);
        // No shared reference over the entire mapping is formed.
        unsafe {
            ptr::copy_nonoverlapping(mmap.as_ptr().add(src), mmap.as_mut_ptr().add(dst), len)
        };
    }

    /// # Safety
    /// Hold the database mutation barrier exclusively.
    pub(crate) unsafe fn flush(&self, ranges: &DirtyRanges) -> Result<()> {
        if !ranges.is_empty() {
            // SAFETY: flush holds the exclusive mutation barrier.
            let mmap = unsafe { self.mapping() };
            for range in ranges.iter() {
                mmap.flush_async_range(range.start, range.len())?;
            }
        }
        self.sync(!ranges.is_empty())
    }

    pub(crate) fn sync(&self, force: bool) -> Result<()> {
        // Callers hold the exclusive mutation barrier. Leave dirty state intact
        // on failure; a clean flush needs no atomic read-modify-write.
        if force || self.dirty.load(Ordering::Relaxed) {
            self.file.sync_all()?;
            self.dirty.store(false, Ordering::Relaxed);
        }
        Ok(())
    }

    pub(crate) fn punch_hole(&self, start: usize, len: usize) -> Result<()> {
        self.dirty.store(true, Ordering::Relaxed);
        punch_hole(&self.file, start, len)
    }
}
