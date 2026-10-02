use std::fs;

use tempfile::TempDir;

use crate::{Database, PAGE_SIZE, Result};

use super::setup_test_db;

#[test]
fn vacant_slots_are_exhausted_before_appending_after_reopen() -> Result<()> {
    let (db, dir) = setup_test_db()?;
    for index in 0..5 {
        let region = db.create_region_if_needed(&format!("old-{index}"))?;
        assert_eq!(region.index(), index);
        region.write(&[index as u8])?;
    }
    db.flush()?;
    let slots = fs::metadata(dir.path().join("regions"))?.len() as usize / PAGE_SIZE;
    assert!(slots > 5, "the fixture must include spare metadata slots");
    drop(db);

    let db = Database::open(dir.path())?;
    for index in (0..5).rev() {
        db.remove_region(&format!("old-{index}"))?;
    }
    for index in 0..slots + 2 {
        let region = db.create_region_if_needed(&format!("new-{index}"))?;
        assert_eq!(region.index(), index);
        region.write(&[index as u8])?;
    }
    db.flush()?;
    drop(db);

    let db = Database::open(dir.path())?;
    for index in 0..5 {
        assert!(db.get_region(&format!("old-{index}")).is_none());
    }
    for index in 0..slots + 2 {
        let region = db.get_region(&format!("new-{index}")).unwrap();
        assert_eq!(region.index(), index);
        assert_eq!(region.create_reader().read_all(), &[index as u8]);
    }
    Ok(())
}

#[test]
fn retention_and_compaction_preserve_relocated_slots() -> Result<()> {
    let (db, dir) = setup_test_db()?;
    let first = db.create_region_if_needed("first")?;
    first.write(b"first")?;
    db.create_region_if_needed("second")?.write(b"second")?;
    db.create_region_if_needed("discarded")?
        .write(b"discarded")?;
    first.reserve_capacity(2 * PAGE_SIZE)?;
    let relocated_start = first.meta().start();
    assert!(relocated_start >= 3 * PAGE_SIZE);
    db.flush()?;
    drop(first);
    drop(db);

    let db = Database::open(dir.path())?;
    assert!(db.get_region("first").is_some());
    assert!(db.get_region("second").is_some());
    db.retain_accessed_regions()?;
    assert_eq!(
        fs::metadata(dir.path().join("regions"))?.len(),
        2 * PAGE_SIZE as u64
    );
    let new = db.create_region_if_needed("new")?;
    assert_eq!(new.index(), 2);
    new.write(b"new")?;
    db.compact()?;
    drop(new);
    drop(db);

    let db = Database::open(dir.path())?;
    assert!(db.get_region("discarded").is_none());
    for (index, id) in ["first", "second", "new"].into_iter().enumerate() {
        let region = db.get_region(id).unwrap();
        assert_eq!(region.index(), index);
        assert_eq!(region.create_reader().read_all(), id.as_bytes());
    }
    assert_eq!(
        db.get_region("first").unwrap().meta().start(),
        relocated_start
    );
    Ok(())
}

#[test]
fn growth_respects_live_pending_and_reusable_tail_allocations() -> Result<()> {
    for (remove_tail, flush, capacity, expected_start) in [
        (false, false, 2 * PAGE_SIZE, 2 * PAGE_SIZE),
        (true, false, 2 * PAGE_SIZE, 2 * PAGE_SIZE),
        (true, true, 2 * PAGE_SIZE, 0),
        (true, true, 3 * PAGE_SIZE, 2 * PAGE_SIZE),
    ] {
        let (db, dir) = setup_test_db()?;
        let first = db.create_region_if_needed("first")?;
        first.write(b"first")?;
        let tail = db.create_region_if_needed("tail")?;
        tail.write(b"tail")?;
        if remove_tail {
            tail.remove()?;
        }
        if flush {
            db.flush()?;
        }

        first.reserve_capacity(capacity)?;
        assert_eq!(first.meta().start(), expected_start);
        assert_eq!(first.meta().reserved(), capacity);
        assert_eq!(first.create_reader().read_all(), b"first");
        db.flush()?;
        drop(first);
        drop(db);

        let db = Database::open(dir.path())?;
        assert_eq!(
            db.get_region("first").unwrap().meta().start(),
            expected_start
        );
        assert_eq!(
            db.get_region("first").unwrap().create_reader().read_all(),
            b"first"
        );
        if remove_tail {
            assert!(db.get_region("tail").is_none());
        } else {
            assert_eq!(
                db.get_region("tail").unwrap().create_reader().read_all(),
                b"tail"
            );
        }
    }
    Ok(())
}

#[test]
fn metadata_slot_reuse_survives_reopen_and_shrink() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    for index in 0..10 {
        db.create_region_if_needed(&index.to_string())?
            .write(&[index])?;
    }
    for index in [0, 2, 4, 6, 8, 9] {
        db.remove_region(&index.to_string())?;
    }
    // Metadata slots are reusable even before removed data holes are flushed.
    let pending = db.create_region_if_needed("pending")?;
    assert_eq!(pending.index(), 0);
    assert_eq!(db.regions().len(), 5);
    assert!(db.get_region("0").is_none());
    assert!(db.get_region("pending").is_some());
    pending.remove()?;
    db.flush()?;
    drop(db);

    let db = Database::open(dir.path())?;
    for index in [1, 3, 5, 7] {
        assert_eq!(
            db.get_region(&index.to_string())
                .unwrap()
                .create_reader()
                .read_all(),
            &[index]
        );
    }
    db.retain_accessed_regions()?;
    for (index, slot) in [0, 2, 4, 6, 8, 9].into_iter().enumerate() {
        let region = db.create_region_if_needed(&format!("new-{index}"))?;
        assert_eq!(region.index(), slot);
        region.write(&[index as u8])?;
    }
    db.flush()?;
    drop(db);
    let db = Database::open(dir.path())?;
    for index in 0..6 {
        assert_eq!(
            db.get_region(&format!("new-{index}"))
                .unwrap()
                .create_reader()
                .read_all(),
            &[index as u8]
        );
    }
    Ok(())
}

