// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    fs::File,
    io::{Error, ErrorKind, Result, Write},
    path::Path,
};

#[cfg(unix)]
use byteview::ByteView;
#[cfg(unix)]
use rustix::io::pread;
#[cfg(windows)]
use std::os::windows::fs::FileExt;

use tempfile::NamedTempFile;

use crate::{FORMAT_VERSION, Slice};

#[cfg(windows)]
use std::{thread, time::Duration};

pub const MAGIC_BYTES: [u8; 4] = [b'L', b'S', b'M', FORMAT_VERSION];

pub const TABLES_FOLDER: &str = "tables";
pub const CURRENT_VERSION_FILE: &str = "current";

/// Reads bytes from a file using `pread`.
pub fn read_exact(file: &File, offset: u64, size: usize) -> Result<Slice> {
    #[cfg(unix)]
    {
        ByteView::try_init(size, |buffer| {
            let (initialized, _) = pread(file, buffer, offset)?;
            check_read(file, offset, size, initialized.len())?;
            Ok(initialized)
        })
        .map(Into::into)
    }

    #[cfg(windows)]
    {
        let mut builder = Slice::builder(size);
        let bytes_read = file.seek_read(&mut builder, offset)?;
        check_read(file, offset, size, bytes_read)?;
        Ok(builder.freeze().into())
    }

    #[cfg(not(any(unix, windows)))]
    compile_error!("unsupported platform");
}

fn check_read(file: &File, offset: u64, size: usize, bytes_read: usize) -> Result<()> {
    if bytes_read != size {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!(
                "read_exact({bytes_read}) at {offset} did not read enough bytes {size}; file has length {}",
                file.metadata()?.len()
            ),
        ));
    }
    Ok(())
}

/// Runs an I/O operation, retrying transient Windows errors with backoff.
///
/// External programs (antivirus, indexers, backup agents) may briefly hold files open
/// without sharing flags on Windows, failing renames and deletes that would succeed a
/// moment later; POSIX has no such failure mode, so elsewhere the operation runs once.
pub fn retry_transient_io<T>(mut op: impl FnMut() -> Result<T>) -> Result<T> {
    #[cfg(windows)]
    {
        const MAX_ATTEMPTS: u32 = 10;

        let mut delay = Duration::from_millis(1);

        for _ in 1..MAX_ATTEMPTS {
            match op() {
                Err(e) if is_transient_windows_error(&e) => {
                    thread::sleep(delay);
                    delay *= 2;
                }
                result => return result,
            }
        }
    }

    op()
}

#[cfg(windows)]
fn is_transient_windows_error(e: &Error) -> bool {
    // ERROR_ACCESS_DENIED, ERROR_SHARING_VIOLATION, ERROR_LOCK_VIOLATION, ERROR_USER_MAPPED_FILE
    matches!(e.raw_os_error(), Some(5 | 32 | 33 | 1224))
}

/// Persists a named temporary file to its final path, replacing any existing file.
pub fn persist_temp_file(temp_file: NamedTempFile, path: &Path) -> Result<()> {
    let mut temp_file = Some(temp_file);

    retry_transient_io(|| {
        #[expect(clippy::expect_used, reason = "the temp file is put back on failure")]
        temp_file
            .take()
            .expect("temp file should be present")
            .persist(path)
            .map(|_| ())
            .map_err(|e| {
                temp_file = Some(e.file);
                e.error
            })
    })
}

/// Atomically rewrites a file.
pub fn rewrite_atomic(path: &Path, content: &[u8]) -> Result<()> {
    #[expect(
        clippy::expect_used,
        reason = "every file should have a parent directory"
    )]
    let folder = path.parent().expect("should have a parent");

    let mut temp_file = NamedTempFile::new_in(folder)?;
    temp_file.write_all(content)?;
    temp_file.flush()?;
    persist_temp_file(temp_file, path)?;

    Ok(())
}
