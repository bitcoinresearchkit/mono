use std::{
    fs::{self, File},
    io::ErrorKind,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::mpsc,
    thread,
    time::Duration,
};

use tempfile::TempDir;

use crate::{Database, Error, PAGE_SIZE, Result};

use super::{allocated_bytes, setup_test_db};

#[test]
fn failed_flush_preserves_dirty_ranges_and_pending_holes() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let removed = db.create_region_if_needed("removed")?;
    removed.write(b"old allocation")?;
    let kept = db.create_region_if_needed("kept")?;
    kept.write(b"kept")?;
    db.flush()?;
    removed.remove()?;
    kept.write_at(b"new!", 0)?;

    // Force the actual mmap flush to fail without OS permissions or test hooks.
    // This intentionally invalid bookkeeping must survive each failed attempt.
    {
        let _access = kept.0.access.write();
        let _writes = db.inner.writes.read();
        // SAFETY: both mutation guards are held; only the test range is invalid.
        unsafe { kept.0.mark_dirty(db.file_len(), 1) };
    }
    for _ in 0..2 {
        assert!(
            matches!(db.flush(), Err(Error::IO(error)) if error.kind() == ErrorKind::InvalidInput)
        );
        assert!(db.layout().start_to_hole().is_empty());
        assert_eq!(kept.create_reader().read_all(), b"new!");
    }
    Ok(())
}

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
fn failed_removal_preserves_region_and_allocation() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let region = db.create_region_if_needed("kept")?;
    region.write(b"kept")?;
    assert!(matches!(
        region.clone().remove(),
        Err(Error::RegionStillReferenced { .. })
    ));
    db.flush()?;
    assert_eq!(db.layout().start_to_region().len(), 1);
    assert!(db.layout().start_to_hole().is_empty());
    db.create_region_if_needed("other")?.write(b"other")?;
    assert_eq!(region.create_reader().read_all(), b"kept");
    drop(region);
    db.remove_region("kept")?;
    Ok(())
}

#[test]
fn invalid_ids_return_errors_without_consuming_holes() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    db.create_region_if_needed("removed")?.remove()?;
    db.flush()?;
    for id in [String::new(), "bad\nid".to_owned(), "x".repeat(1025)] {
        let result = catch_unwind(AssertUnwindSafe(|| db.create_region_if_needed(&id)));
        assert!(matches!(result, Ok(Err(Error::InvalidRegionId))));
        assert_eq!(db.layout().start_to_hole().get(&0), Some(&PAGE_SIZE));
        assert_eq!(db.regions().len(), 0);
    }
    Ok(())
}

#[test]
fn oversized_reservation_returns_error_without_changing_region() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let region = db.create_region_if_needed("kept")?;
    region.write(b"kept")?;
    for size in [usize::MAX, (1usize << 40) + PAGE_SIZE] {
        let result = catch_unwind(AssertUnwindSafe(|| region.reserve_capacity(size)));
        assert!(matches!(result, Ok(Err(Error::RegionSizeOverflow { .. }))));
        assert_eq!(region.meta().reserved(), PAGE_SIZE);
        assert_eq!(region.create_reader().read_all(), b"kept");
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
fn open_rejects_empty_and_control_character_ids() -> Result<()> {
    for id in [b"".as_slice(), b"bad\nid"] {
        let dir = TempDir::new()?;
        let db = Database::open(dir.path())?;
        let _ = db.create_region_if_needed("valid")?;
        db.flush()?;
        drop(db);
        let path = dir.path().join("regions");
        let mut bytes = fs::read(&path)?;
        bytes[24..32].copy_from_slice(&(id.len() as u64).to_le_bytes());
        bytes[32..32 + id.len()].copy_from_slice(id);
        fs::write(path, bytes)?;
        assert!(matches!(
            Database::open(dir.path()),
            Err(Error::CorruptedMetadata(_))
        ));
    }
    Ok(())
}

#[test]
fn batch_panic_preserves_dirty_tracking() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let region = db.create_region_if_needed("batch")?;
    region.write(&[0; 16])?;
    db.flush()?;
    let result = catch_unwind(AssertUnwindSafe(|| {
        region.batch_write_ordered([(0, 7u8), (8, 9)].into_iter(), 1, |value, bytes| {
            bytes[0] = *value;
            if *value == 9 {
                panic!("callback failed after writing");
            }
        });
    }));
    assert!(result.is_err());
    assert_eq!(db.flush()?, 1);
    assert_eq!(region.create_reader().read(8, 1), &[9]);
    Ok(())
}

#[test]
fn batch_iterator_panic_preserves_completed_writes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("batch")?;
    region.write(&[0; 16])?;
    db.flush()?;
    let result = catch_unwind(AssertUnwindSafe(|| {
        let values = (0..3).map(|index| {
            assert_ne!(index, 1, "iterator failed after its first item");
            (index * 4, 9u8)
        });
        region.batch_write_ordered(values, 1, |value, bytes| bytes[0] = *value);
    }));
    assert!(result.is_err());
    assert_eq!(db.flush()?, 1);
    assert_eq!(region.create_reader().read(0, 1), &[9]);
    assert_eq!(region.create_reader().read(8, 1), &[0]);
    assert_eq!(db.flush()?, 0);
    Ok(())
}

#[test]
fn invalid_batch_bounds_leave_data_clean() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("batch")?;
    region.write(&[1; 16])?;
    db.flush()?;
    let cases: &[(&[usize], usize)] = &[
        (&[8, 0], 1),
        (&[0, 16], 8),
        (&[0, usize::MAX], 8),
        (&[0], 17),
        (&[0], usize::MAX),
    ];
    for &(offsets, value_len) in cases {
        let mut called = false;
        let result = catch_unwind(AssertUnwindSafe(|| {
            region.batch_write_ordered(
                offsets.iter().map(|&offset| (offset, ())),
                value_len,
                |_, bytes| {
                    called = true;
                    bytes.fill(7);
                },
            );
        }));
        assert!(result.is_err());
        assert!(!called);
        assert_eq!(db.flush()?, 0);
        assert_eq!(region.create_reader().read_all(), &[1; 16]);
    }
    Ok(())
}

#[test]
fn middle_batch_offsets_cannot_escape_the_dirty_span() -> Result<()> {
    for invalid in [0, 16, usize::MAX] {
        let (db, temp) = setup_test_db()?;
        let region = db.create_region_if_needed("batch")?;
        region.write(&[0; 24])?;
        db.flush()?;
        let result = catch_unwind(AssertUnwindSafe(|| {
            region.batch_write_ordered(
                [(4, 7u8), (invalid, 9), (8, 11)].into_iter(),
                1,
                |value, bytes| bytes[0] = *value,
            );
        }));
        assert!(result.is_err());
        assert_eq!(db.flush()?, 1);
        drop(region);
        drop(db);
        let db = Database::open(temp.path())?;
        let mut expected = [0; 24];
        expected[4] = 7;
        assert_eq!(
            db.get_region("batch").unwrap().create_reader().read_all(),
            expected
        );
    }
    Ok(())
}

#[test]
fn background_panic_returns_error_and_joins_other_tasks() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let (done, finished) = mpsc::channel();
    db.run_bg(|_| panic!("background failure"));
    db.run_bg(move |db| {
        db.bg_sleep(Duration::from_secs(5));
        done.send(()).unwrap();
        Ok(())
    });
    assert!(matches!(
        db.sync_bg_tasks(),
        Err(Error::BackgroundTaskPanicked)
    ));
    finished.try_recv().unwrap();
    // The join state is reset even on failure.
    db.run_bg(|db| db.flush().map(|_| ()));
    db.sync_bg_tasks()?;
    Ok(())
}

#[test]
fn database_growth_overflow_returns_error() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    assert!(matches!(
        db.set_min_len(usize::MAX),
        Err(Error::FileSizeOverflow { .. })
    ));
    db.create_region_if_needed("valid")?
        .write(b"still usable")?;
    Ok(())
}

#[test]
#[should_panic]
fn read_cannot_escape_region_or_wrap() {
    let dir = TempDir::new().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let first = db.create_region_if_needed("first").unwrap();
    let second = db.create_region_if_needed("second").unwrap();
    first.write(b"first").unwrap();
    second.write(b"second").unwrap();
    second.create_reader().read(usize::MAX, 1);
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

#[test]
fn retaining_no_regions_shrinks_metadata_to_zero_and_allows_reuse() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    db.create_region_if_needed("old")?.write(b"old")?;
    db.flush()?;
    drop(db);
    let db = Database::open(dir.path())?;
    db.retain_accessed_regions()?;
    assert_eq!(fs::metadata(dir.path().join("regions"))?.len(), 0);
    db.create_region_if_needed("new")?.write(b"new")?;
    db.flush()?;
    drop(db);
    let db = Database::open(dir.path())?;
    assert_eq!(
        db.get_region("new").unwrap().create_reader().read_all(),
        b"new"
    );
    Ok(())
}
