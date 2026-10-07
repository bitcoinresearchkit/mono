#[cfg(target_os = "macos")]
use std::os::fd::AsRawFd;
use std::{
    fs::File,
    path::Path,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};
#[cfg(unix)]
use std::{io, os::unix::fs::FileExt};

use memmap2::MmapRaw;
use parking_lot::{RwLock, RwLockReadGuard};

use crate::{Error, PAGE_SIZE, Result};

use super::{hole_punch::punch_hole, locked_file::open_locked_file, mmap::write_to_mmap};

/// Owns the data file and its mapping, plus, on macOS, a second descriptor that bypasses the page cache.
pub(crate) struct DataFile {
    file: File,
    /// On macOS dirtying a cached page (through the mapping or pwrite) costs about 100 us per 16 KiB
    /// page, while uncached reads and writes keep the SSD busy in parallel. An uncached write drops the
    /// cached copy; uncached reads see unflushed writes through the mapping.
    #[cfg(target_os = "macos")]
    uncached: File,
    mmap: RwLock<MmapRaw>,
    len: AtomicUsize,
}

impl DataFile {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let file = open_locked_file(path)?;
        let len = usize::try_from(file.metadata()?.len()).map_err(|_| Error::FileSizeOverflow {
            requested: usize::MAX,
        })?;
        let mmap = MmapRaw::map_raw(&file)?;
        #[cfg(target_os = "macos")]
        let uncached = {
            let uncached = File::options().read(true).write(true).open(path)?;
            // SAFETY: an advisory flag on an open descriptor; a failure only leaves caching on.
            unsafe { libc::fcntl(uncached.as_raw_fd(), libc::F_NOCACHE, 1) };
            uncached
        };
        Ok(Self {
            file,
            #[cfg(target_os = "macos")]
            uncached,
            mmap: RwLock::new(mmap),
            len: AtomicUsize::new(len),
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

    /// Reads through the uncached descriptor. Callers hold the mutation barrier and exclusive access to
    /// the range.
    #[cfg(target_os = "macos")]
    pub(crate) fn read_uncached(&self, start: usize, buf: &mut [u8]) -> io::Result<()> {
        self.uncached.read_exact_at(buf, start as u64)
    }

    /// Writes through the uncached descriptor. Callers hold the mutation barrier and exclusive access to
    /// the range.
    #[cfg(target_os = "macos")]
    pub(crate) fn write_uncached(&self, start: usize, buf: &[u8]) -> io::Result<()> {
        self.uncached.write_all_at(buf, start as u64)
    }

    /// Reads through the cached descriptor, leaving the range in the page cache. Callers hold the
    /// mutation barrier and exclusive access to the range.
    #[cfg(unix)]
    pub(crate) fn read_cached(&self, start: usize, buf: &mut [u8]) -> io::Result<()> {
        self.file.read_exact_at(buf, start as u64)
    }

    /// Drops the cached pages of a page-aligned range, so the next access reads the file. Callers hold
    /// the mutation barrier and exclusive access to the range, and no page in it is dirty.
    #[cfg(target_os = "macos")]
    pub(crate) fn invalidate(&self, start: usize, len: usize) -> io::Result<()> {
        // SAFETY: the barrier keeps the mapping in place; the caller's range lies within it, and
        // invalidating clean pages only drops cached copies of what is on disk.
        let result = unsafe {
            libc::msync(
                self.mapping().as_mut_ptr().add(start).cast(),
                len,
                libc::MS_INVALIDATE,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub(crate) fn punch_hole(&self, start: usize, len: usize) -> Result<()> {
        punch_hole(&self.file, start, len)
    }
}
