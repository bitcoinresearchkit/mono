//! Critical storage invariants: relocation, compaction and slot reuse across reopen, concurrent relocation with
//! flushes, and writer exclusion.

use std::{
    fs::{self, File},
    sync::Barrier,
    thread,
};

use tempfile::TempDir;

use crate::{Database, Error, PAGE_SIZE, Result};

fn setup_test_db() -> Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db = Database::open(temp_dir.path())?;
    Ok((db, temp_dir))
}

#[test]
fn truncating_relocation_preserves_prefix_and_neighbor_on_reopen() -> Result<()> {
    for prefix in [0, 17, PAGE_SIZE] {
        let (db, dir) = setup_test_db()?;
        let region = db.create_region_if_needed("rewrite")?;
        region.write(&vec![3; 2 * PAGE_SIZE])?;
        let old_start = region.meta().start();
        let neighbor = db.create_region_if_needed("neighbor")?;
        neighbor.write(b"kept")?;
        db.flush();

        region.truncate_write(prefix, &vec![7; 3 * PAGE_SIZE - prefix])?;
        assert_ne!(region.meta().start(), old_start);
        db.flush();
        drop(region);
        drop(neighbor);
        drop(db);

        let db = Database::open(dir.path())?;
        let region = db.get_region("rewrite").unwrap();
        let reader = region.create_reader();
        assert_eq!(reader.len(), 3 * PAGE_SIZE);
        assert!(reader.read_all()[..prefix].iter().all(|&byte| byte == 3));
        assert!(reader.read_all()[prefix..].iter().all(|&byte| byte == 7));
        assert_eq!(
            db.get_region("neighbor")
                .unwrap()
                .create_reader()
                .read_all(),
            b"kept"
        );
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
    db.flush();
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
    db.flush();
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
    db.flush();
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

#[test]
fn concurrent_relocations_and_flushes_preserve_every_region() -> Result<()> {
    let dir = TempDir::new()?;
    {
        let db = Database::open(dir.path())?;
        db.set_min_len(16 * 1024 * 1024)?;
        let mut regions = Vec::new();
        for id in 0..4u8 {
            let region = db.create_region_if_needed(&id.to_string())?;
            region.write(&vec![id; 1024 * 1024])?;
            regions.push(region);
        }
        // Block in-place growth, so each writer must copy to a new allocation.
        drop(db.create_region_if_needed("blocker")?);
        db.flush();
        let barrier = Barrier::new(regions.len() + 1);
        thread::scope(|scope| -> Result<()> {
            let handles: Vec<_> = regions
                .iter()
                .map(|region| {
                    let barrier = &barrier;
                    scope.spawn(move || {
                        barrier.wait();
                        region.write(&[9])
                    })
                })
                .collect();
            barrier.wait();
            db.flush();
            for handle in handles {
                handle.join().unwrap()?;
            }
            Ok(())
        })?;
        db.flush();
    }
    let db = Database::open(dir.path())?;
    for id in 0..4u8 {
        let reader = db.get_region(&id.to_string()).unwrap().create_reader();
        assert_eq!(reader.len(), 1024 * 1024 + 1);
        assert!(reader.read_all()[..1024 * 1024].iter().all(|&b| b == id));
        assert_eq!(reader.read_all().last(), Some(&9));
    }
    Ok(())
}

#[test]
fn opening_either_locked_file_preserves_existing_data() -> Result<()> {
    for name in ["data", "regions"] {
        let dir = TempDir::new()?;
        let db = Database::open(dir.path())?;
        db.create_region_if_needed("kept")?.write(b"kept")?;
        db.flush();
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
