use brk_error::{Error, Result};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::{
    fs::{self, DirEntry, Metadata},
    io::{Error as IoError, ErrorKind},
    path::Path,
};

// Bound recursive frames even if directories or links change during a walk.
const MAX_DEPTH: usize = 128;

/// Allocated bytes under `path`, leaving out the directories in `skip` and everything in them.
/// Paths are compared as written: `skip` and `path` must be spelled from the same root.
pub(crate) fn dir_size(path: &Path, skip: &[&Path]) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        dir_contents_size(path, None, skip)
    } else {
        allocated_bytes(&metadata)
    }
}

struct DirectoryFrame<'a> {
    path: &'a Path,
    parent: Option<&'a DirectoryFrame<'a>>,
    depth: usize,
}

impl DirectoryFrame<'_> {
    #[cfg(unix)]
    fn contains(&self, _: &Path, target: &Metadata) -> Result<bool> {
        let mut ancestor = Some(self);
        while let Some(directory) = ancestor {
            let metadata = fs::metadata(directory.path)?;
            if metadata.dev() == target.dev() && metadata.ino() == target.ino() {
                return Ok(true);
            }
            ancestor = directory.parent;
        }
        Ok(false)
    }

    #[cfg(not(unix))]
    fn contains(&self, path: &Path, _: &Metadata) -> Result<bool> {
        let target = fs::canonicalize(path)?;
        let mut ancestor = Some(self);
        while let Some(directory) = ancestor {
            if fs::canonicalize(directory.path)? == target {
                return Ok(true);
            }
            ancestor = directory.parent;
        }
        Ok(false)
    }
}

fn dir_contents_size(
    path: &Path,
    parent: Option<&DirectoryFrame<'_>>,
    skip: &[&Path],
) -> Result<u64> {
    if skip.contains(&path) {
        return Ok(0);
    }
    let depth = parent.map_or(0, |parent| parent.depth + 1);
    if depth > MAX_DEPTH {
        return Err(IoError::new(
            ErrorKind::InvalidData,
            "disk scan directory depth limit exceeded",
        )
        .into());
    }
    let frame = DirectoryFrame {
        path,
        parent,
        depth,
    };
    // One entry after the other: a few hundred of them cost less than handing them to a pool.
    fs::read_dir(path)?.try_fold(0, |total, entry| {
        sum_bytes(total, or_gone(dir_entry_size(entry?, &frame, skip))?)
    })
}

fn dir_entry_size(
    entry: DirEntry,
    frame: &DirectoryFrame<'_>,
    skip: &[&Path],
) -> Result<u64> {
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
        return dir_contents_size(&entry.path(), Some(frame), skip);
    }

    if file_type.is_symlink() {
        let path = entry.path();
        let metadata = fs::metadata(&path)?;
        return if metadata.is_dir() {
            if frame.contains(&path, &metadata)? {
                return Err(IoError::new(ErrorKind::InvalidData, "directory symlink cycle").into());
            }
            dir_contents_size(&path, Some(frame), skip)
        } else {
            allocated_bytes(&metadata)
        };
    }

    allocated_bytes(&entry.metadata()?)
}

/// Files come and go under a running daemon (rollback files, rotated tips, a temporary being
/// renamed): one listed and gone before it is read weighs nothing.
fn or_gone(size: Result<u64>) -> Result<u64> {
    match size {
        Err(Error::IO(error)) if error.kind() == ErrorKind::NotFound => Ok(0),
        size => size,
    }
}

pub(crate) fn sum_bytes(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "disk byte total overflow").into())
}

#[cfg(unix)]
fn blocks_to_bytes(blocks: u64) -> Result<u64> {
    blocks
        .checked_mul(512)
        .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "allocated byte count overflow").into())
}

#[cfg(unix)]
fn allocated_bytes(metadata: &Metadata) -> Result<u64> {
    // POSIX st_blocks units are always 512 bytes.
    blocks_to_bytes(metadata.blocks())
}

#[cfg(not(unix))]
fn allocated_bytes(metadata: &Metadata) -> Result<u64> {
    Ok(metadata.len())
}
