#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::{
    fs::File,
    path::Path,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};
#[cfg(unix)]
use std::{io, os::unix::fs::FileExt};

use memmap2::{MmapOptions, MmapRaw};
use parking_lot::Mutex;

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
    /// Maps the file's reserved address space once: growing the file inside it needs no remap, so
    /// the mapping never moves.
    mmap: MmapRaw,
    len: AtomicUsize,
    grow: Mutex<()>,
}

impl DataFile {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let file = open_locked_file(path)?;
        let len = usize::try_from(file.metadata()?.len()).map_err(|_| Error::FileSizeOverflow {
            requested: usize::MAX,
        })?;
        let mmap = MmapOptions::new()
            .len(Self::reservation(len))
            .map_raw(&file)?;
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
            mmap,
            len: AtomicUsize::new(len),
            grow: Mutex::default(),
        })
    }

    /// Address space reserved for a file of `len` bytes: four times its length, at least 1 TiB.
    /// Pages past the file's end are mapped but never touched until the file grows over them.
    fn reservation(len: usize) -> usize {
        len.saturating_mul(4)
            .max(1 << 40)
            .next_multiple_of(PAGE_SIZE)
    }

    pub(crate) fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    /// Grows the file to at least `len` bytes inside its reserved mapping, so it never waits for
    /// readers: a writer may grow while holding locks readers wait on. Past the reservation (four
    /// times the file at open, at least 1 TiB) it fails; reopening reserves more.
    pub(crate) fn ensure_len(&self, len: usize) -> Result<()> {
        let len = Self::aligned_len(len)?;
        if self.len() >= len {
            return Ok(());
        }
        let reserved = self.mmap.len();
        if len > reserved {
            return Err(Error::ReservationExhausted {
                requested: len,
                reserved,
            });
        }
        let _grow = self.grow.lock();
        let current_len = self.len();
        if current_len >= len {
            return Ok(());
        }
        let target_len =
            Self::aligned_len(len.max(current_len.saturating_mul(2)).max(1024 * 1024))?
                .min(reserved);
        self.file.set_len(target_len as u64)?;
        self.len.store(target_len, Ordering::Relaxed);
        Ok(())
    }

    fn aligned_len(len: usize) -> Result<usize> {
        len.checked_next_multiple_of(PAGE_SIZE)
            .filter(|&len| len <= isize::MAX as usize)
            .ok_or(Error::FileSizeOverflow { requested: len })
    }

    /// The file's mapping: it never moves while the file is open.
    pub(crate) fn mapping(&self) -> &MmapRaw {
        &self.mmap
    }

    #[cfg(unix)]
    pub(crate) fn advise_read(&self, offset: usize, len: usize) {
        advise_read(&self.file, offset as u64, len);
    }

    /// # Safety
    /// Hold the mutation barrier and exclusive access to the destination bytes.
    pub(crate) unsafe fn write(&self, start: usize, bytes: &[u8]) {
        unsafe { write_to_mmap(&self.mmap, start, bytes) };
    }

    /// # Safety
    /// Hold the mutation barrier and exclusively own the source and destination, two disjoint
    /// ranges within the mapping.
    pub(crate) unsafe fn copy(&self, src: usize, dst: usize, len: usize) {
        let mmap = &self.mmap;
        debug_assert!(src <= mmap.len() && len <= mmap.len() - src);
        debug_assert!(dst <= mmap.len() && len <= mmap.len() - dst);
        debug_assert!(src + len <= dst || dst + len <= src);
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
        // SAFETY: the caller's range lies within the mapping, and invalidating clean pages only
        // drops cached copies of what is on disk.
        let result = unsafe {
            libc::msync(
                self.mmap.as_mut_ptr().add(start).cast(),
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

/// Asks the kernel to start reading a range of `file` into the page cache, without waiting for
/// it. On Linux `posix_fadvise(WILLNEED)` is the same call as macOS's `F_RDADVISE`: file offsets
/// on the descriptor, queued read-ahead, no mapping touched. Several ranges advised before any
/// is read reach the device as one batch rather than one read at a time.
#[cfg(unix)]
pub fn advise_read(file: &File, offset: u64, len: usize) {
    #[cfg(target_os = "macos")]
    let error = {
        // SAFETY: zero is valid for both integer fields and the padding.
        let mut request: libc::radvisory = unsafe { std::mem::zeroed() };
        request.ra_offset = offset as libc::off_t;
        request.ra_count = len.min(libc::c_int::MAX as usize) as libc::c_int;
        // SAFETY: the descriptor stays open and fcntl copies the request.
        let result = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_RDADVISE, &raw const request) };
        (result < 0).then(io::Error::last_os_error)
    };
    #[cfg(not(target_os = "macos"))]
    let error = {
        // SAFETY: the descriptor stays open; the call only reads its arguments.
        let result = unsafe {
            libc::posix_fadvise(
                file.as_raw_fd(),
                offset as libc::off_t,
                len as libc::off_t,
                libc::POSIX_FADV_WILLNEED,
            )
        };
        (result != 0).then(|| io::Error::from_raw_os_error(result))
    };
    // A hint, not a read: when it fails, the reads that follow fetch the same bytes one at a
    // time, so the answer is unchanged and only slower. Logged, never returned.
    if let Some(error) = error {
        log::debug!("read advice at {offset}+{len}: {error}");
    }
}
