use brk_error::Result;
use rayon::iter::{ParallelBridge, ParallelIterator};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::{
    fs::{self, DirEntry, Metadata},
    io::{Error as IoError, ErrorKind},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

// Bound recursive frames even if directories or links change during a walk.
const MAX_DEPTH: usize = 128;

fn check_cancelled(cancelled: &AtomicBool) -> Result<()> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(IoError::new(ErrorKind::Interrupted, "disk scan cancelled").into());
    }
    Ok(())
}

pub fn dir_size(path: &Path, cancelled: &AtomicBool) -> Result<u64> {
    check_cancelled(cancelled)?;
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        dir_contents_size(path, None, cancelled)
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
    fn contains(&self, _: &Path, target: &Metadata, cancelled: &AtomicBool) -> Result<bool> {
        let mut ancestor = Some(self);
        while let Some(directory) = ancestor {
            check_cancelled(cancelled)?;
            let metadata = fs::metadata(directory.path)?;
            if metadata.dev() == target.dev() && metadata.ino() == target.ino() {
                return Ok(true);
            }
            ancestor = directory.parent;
        }
        Ok(false)
    }

    #[cfg(not(unix))]
    fn contains(&self, path: &Path, _: &Metadata, cancelled: &AtomicBool) -> Result<bool> {
        check_cancelled(cancelled)?;
        let target = fs::canonicalize(path)?;
        let mut ancestor = Some(self);
        while let Some(directory) = ancestor {
            check_cancelled(cancelled)?;
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
    cancelled: &AtomicBool,
) -> Result<u64> {
    check_cancelled(cancelled)?;
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
    fs::read_dir(path)?
        .par_bridge()
        .map(|entry| dir_entry_size(entry?, &frame, cancelled))
        .try_reduce(|| 0, sum_bytes)
}

fn dir_entry_size(
    entry: DirEntry,
    frame: &DirectoryFrame<'_>,
    cancelled: &AtomicBool,
) -> Result<u64> {
    check_cancelled(cancelled)?;
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
        return dir_contents_size(&entry.path(), Some(frame), cancelled);
    }

    if file_type.is_symlink() {
        let path = entry.path();
        let metadata = fs::metadata(&path)?;
        return if metadata.is_dir() {
            if frame.contains(&path, &metadata, cancelled)? {
                return Err(IoError::new(ErrorKind::InvalidData, "directory symlink cycle").into());
            }
            dir_contents_size(&path, Some(frame), cancelled)
        } else {
            allocated_bytes(&metadata)
        };
    }

    allocated_bytes(&entry.metadata()?)
}

fn sum_bytes(left: u64, right: u64) -> Result<u64> {
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
