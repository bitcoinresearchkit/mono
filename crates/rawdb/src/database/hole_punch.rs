use std::fs::File;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "freebsd"))]
use std::{io::Error as IoError, os::fd::AsRawFd};

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "freebsd"))]
use libc::off_t;
#[cfg(target_os = "macos")]
use libc::{F_PUNCHHOLE, fcntl, fpunchhole_t};
#[cfg(target_os = "linux")]
use libc::{FALLOC_FL_KEEP_SIZE, FALLOC_FL_PUNCH_HOLE, fallocate};
#[cfg(target_os = "freebsd")]
use libc::{SPACECTL_DEALLOC, fspacectl, spacectl_range};

use crate::{Error, Result};

/// Deallocates file blocks without changing file size. The caller owns the
/// allocation lock and supplies a page-aligned range within the mapped file.
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "freebsd"))]
pub(crate) fn punch_hole(file: &File, start: usize, length: usize) -> Result<()> {
    #[cfg(target_os = "macos")]
    let result = {
        let range = fpunchhole_t {
            fp_flags: 0,
            reserved: 0,
            fp_offset: start as off_t,
            fp_length: length as off_t,
        };
        // SAFETY: range has libc's ABI layout and remains valid during the call.
        unsafe { fcntl(file.as_raw_fd(), F_PUNCHHOLE, &raw const range) }
    };
    #[cfg(target_os = "linux")]
    // SAFETY: the file descriptor is live and the range fits off_t.
    let result = unsafe {
        fallocate(
            file.as_raw_fd(),
            FALLOC_FL_PUNCH_HOLE | FALLOC_FL_KEEP_SIZE,
            start as off_t,
            length as off_t,
        )
    };
    #[cfg(target_os = "freebsd")]
    let result = {
        let mut range = spacectl_range {
            r_offset: start as off_t,
            r_len: length as off_t,
        };
        // SAFETY: fspacectl explicitly allows the request and remaining ranges
        // to share a structure. It remains valid throughout the call.
        unsafe {
            fspacectl(
                file.as_raw_fd(),
                SPACECTL_DEALLOC,
                &raw const range,
                0,
                &raw mut range,
            )
        }
    };

    if result == -1 {
        return Err(Error::HolePunchFailed {
            start,
            len: length,
            source: IoError::last_os_error(),
        });
    }
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "freebsd")))]
pub(crate) fn punch_hole(_file: &File, _start: usize, _length: usize) -> Result<()> {
    Err(Error::HolePunchUnsupported)
}
