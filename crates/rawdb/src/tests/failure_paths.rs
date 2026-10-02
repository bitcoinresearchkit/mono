use std::{
    fs::{self, File},
    sync::mpsc,
    thread,
    time::Duration,
};

use tempfile::TempDir;

use crate::{Database, Error, PAGE_SIZE, Result};

use super::allocated_bytes;

#[test]
fn opening_either_locked_file_preserves_existing_data() -> Result<()> {
    for name in ["data", "regions"] {
        let dir = TempDir::new()?;
        let db = Database::open(dir.path())?;
        db.create_region_if_needed("kept")?.write(b"kept")?;
        db.flush()?;
        drop(db);

        let path = dir.path().join(name);
        let before = fs::read(&path)?;
        let file = File::options().read(true).write(true).open(&path)?;
        file.try_lock()?;
        assert!(matches!(Database::open(dir.path()), Err(Error::TryLock(_))));
        assert_eq!(fs::read(&path)?, before);
        drop(file);

        let db = Database::open(dir.path())?;
        assert_eq!(
            db.get_region("kept").unwrap().create_reader().read_all(),
            b"kept"
        );
    }
    Ok(())
}

#[test]
fn background_error_still_joins_remaining_tasks() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let (done, finished) = mpsc::channel();
    db.run_bg(|_| Err(Error::other("first task failed")));
    db.run_bg(move |_| {
        thread::sleep(Duration::from_millis(30));
        done.send(()).unwrap();
        Ok(())
    });
    let result = db.sync_bg_tasks();
    assert_eq!(result.unwrap_err().to_string(), "first task failed");
    finished
        .try_recv()
        .expect("sync returned before the remaining task completed");
    Ok(())
}

#[test]
fn compaction_reclaims_removed_file_tail_after_reopen() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    db.create_region_if_needed("kept")?.write(b"kept")?;
    let removed = db.create_region_if_needed("removed")?;
    removed.write(&vec![1; 64 * PAGE_SIZE])?;
    db.flush()?;
    removed.remove()?;
    db.flush()?;
    let before = allocated_bytes(&db)?;
    drop(db);
    let db = Database::open(dir.path())?;
    db.compact()?;
    assert!(allocated_bytes(&db)? < before);
    assert_eq!(
        db.get_region("kept").unwrap().create_reader().read_all(),
        b"kept"
    );
    Ok(())
}
