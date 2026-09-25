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

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
    };

    use tempfile::tempdir;
    use test_log::test;

    use super::*;
    use crate::Result as CrateResult;

    #[test]
    fn positional_read_initialization_and_short_reads() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        let bytes: Vec<_> = (0..=255_u8).cycle().take(8192).collect();
        file.write_all(&bytes)?;
        for len in [0, 8, 12, 13, 4096] {
            assert_eq!(
                read_exact(file.as_file(), 7, len)?.as_ref(),
                &bytes[7..7 + len]
            );
        }
        let result = read_exact(file.as_file(), 8190, 16);
        assert!(matches!(result, Err(error) if error.kind() == ErrorKind::UnexpectedEof));
        Ok(())
    }

    #[test]
    fn atomic_rewrite() -> CrateResult<()> {
        let dir = tempdir()?;

        let path = dir.path().join("test.txt");
        {
            let mut file = File::create(&path)?;
            write!(file, "asdasdasdasdasd")?;
        }

        rewrite_atomic(&path, b"newcontent")?;

        let content = fs::read_to_string(&path)?;
        assert_eq!("newcontent", content);

        Ok(())
    }

    #[test]
    fn persist_temp_file_replaces_existing() -> CrateResult<()> {
        let dir = tempdir()?;

        let path = dir.path().join("test.txt");
        fs::write(&path, b"old")?;

        let mut temp_file = NamedTempFile::new_in(dir.path())?;
        write!(temp_file, "new")?;
        persist_temp_file(temp_file, &path)?;

        let content = fs::read_to_string(&path)?;
        assert_eq!("new", content);

        Ok(())
    }

    #[test]
    #[cfg(windows)]
    fn retry_transient_io_retries_sharing_violation() {
        let mut attempts = 0;

        let result = retry_transient_io(|| {
            attempts += 1;

            if attempts < 3 {
                // ERROR_SHARING_VIOLATION
                Err(Error::from_raw_os_error(32))
            } else {
                Ok(42)
            }
        });

        assert!(matches!(result, Ok(42)));
        assert_eq!(3, attempts);
    }

    #[test]
    #[cfg(windows)]
    fn retry_transient_io_fails_fast_on_other_errors() {
        let mut attempts = 0;

        let result: Result<()> = retry_transient_io(|| {
            attempts += 1;

            // ERROR_FILE_NOT_FOUND
            Err(Error::from_raw_os_error(2))
        });

        assert!(result.is_err());
        assert_eq!(1, attempts);
    }
}
