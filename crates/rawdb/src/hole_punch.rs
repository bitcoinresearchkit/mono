use std::fs::File;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

use crate::Error;

/// Deallocates file blocks without changing file size (platform-specific).
pub struct HolePunch;

impl HolePunch {
    #[cfg(target_os = "macos")]
    pub fn punch(file: &File, start: usize, length: usize) -> crate::Result<()> {
        let fpunchhole = FPunchhole {
            fp_flags: 0,
            reserved: 0,
            fp_offset: start as libc::off_t,
            fp_length: length as libc::off_t,
        };

        let result = unsafe {
            libc::fcntl(
                file.as_raw_fd(),
                libc::F_PUNCHHOLE,
                &fpunchhole as *const FPunchhole,
            )
        };

        if result == -1 {
            let err = std::io::Error::last_os_error();
            return Err(Error::HolePunchFailed {
                start,
                len: length,
                source: err,
            });
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn punch(file: &File, start: usize, length: usize) -> crate::Result<()> {
        let result = unsafe {
            libc::fallocate(
                file.as_raw_fd(),
                libc::FALLOC_FL_PUNCH_HOLE | libc::FALLOC_FL_KEEP_SIZE,
                start as libc::off_t,
                length as libc::off_t,
            )
        };

        if result == -1 {
            let err = std::io::Error::last_os_error();
            return Err(Error::HolePunchFailed {
                start,
                len: length,
                source: err,
            });
        }

        Ok(())
    }

    #[cfg(target_os = "freebsd")]
    pub fn punch(file: &File, start: usize, length: usize) -> crate::Result<()> {
        let fd = file.as_raw_fd();

        let mut spacectl = libc::spacectl_range {
            r_offset: start as libc::off_t,
            r_len: length as libc::off_t,
        };

        let result = unsafe {
            libc::fspacectl(
                fd,
                libc::SPACECTL_DEALLOC,
                &spacectl as *const libc::spacectl_range,
                0,
                &mut spacectl as *mut libc::spacectl_range,
            )
        };

        if result == -1 {
            let err = std::io::Error::last_os_error();
            return Err(Error::HolePunchFailed {
                start,
                len: length,
                source: err,
            });
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "freebsd")))]
    pub fn punch(_file: &File, _start: usize, _length: usize) -> crate::Result<()> {
        Err(Error::HolePunchUnsupported)
    }
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct FPunchhole {
    fp_flags: u32,
    reserved: u32,
    fp_offset: libc::off_t,
    fp_length: libc::off_t,
}
