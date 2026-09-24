mod allocation;
mod compaction;
mod concurrency;
mod database;
mod dirty_ranges;
mod failure_paths;
mod indexed_writes;
mod lifecycle;
mod metadata;
mod persistence;
mod reader;
#[cfg(unix)]
mod residency;
mod writes;

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

use tempfile::TempDir;

use crate::{Database, Result};

fn setup_test_db() -> Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db = Database::open(temp_dir.path())?;
    Ok((db, temp_dir))
}

fn allocated_bytes(db: &Database) -> Result<u64> {
    let metadata = fs::metadata(db.path().join("data"))?;
    #[cfg(unix)]
    return Ok(metadata.blocks() * 512);
    #[cfg(not(unix))]
    Ok(metadata.len())
}
