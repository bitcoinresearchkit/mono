//! Critical storage invariants: relocation, compaction and slot reuse across reopen, concurrent relocation with
//! flushes, writer exclusion, and indexed writes through the uncached descriptor.

use std::{
    collections::BTreeMap,
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

#[test]
fn indexed_writes_match_the_mapping_across_pages_neighbors_and_reopen() -> Result<()> {
    const LEN: usize = 37;
    const AT: usize = 11;
    const COUNT: usize = 60_000;
    let (db, dir) = setup_test_db()?;
    // Small neighbors share 16 KiB pages with the region; its interior pages are its own.
    let before = db.create_region_if_needed("before")?;
    before.write(&[1; PAGE_SIZE])?;
    let region = db.create_region_if_needed("region")?;
    let mut expected = vec![0u8; AT + COUNT * LEN];
    region.write(&expected)?;
    let after = db.create_region_if_needed("after")?;
    after.write(&[2; PAGE_SIZE])?;
    let write = |values: BTreeMap<usize, [u8; LEN]>, expected: &mut Vec<u8>| -> Result<()> {
        for (&i, value) in &values {
            expected[AT + i * LEN..AT + (i + 1) * LEN].copy_from_slice(value);
        }
        region.write_indexed(values, LEN, AT, |value, bytes| bytes.copy_from_slice(value))?;
        assert_eq!(region.create_reader().read_all(), &expected[..]);
        Ok(())
    };

    // Cold pages: write them back and drop them (macOS evicts on MS_INVALIDATE; Linux keeps them).
    #[cfg(unix)]
    {
        let _writes = db.inner.writes.write();
        // SAFETY: the exclusive barrier keeps the mapping in place; msync only flushes and drops pages.
        let mapping = unsafe { db.inner.data.mapping() };
        let evict =
            |flags| unsafe { libc::msync(mapping.as_mut_ptr().cast(), mapping.len(), flags) };
        assert_eq!(evict(libc::MS_SYNC), 0);
        assert_eq!(evict(libc::MS_INVALIDATE), 0);
        // Linux keeps pages on MS_INVALIDATE; drop the now clean pages explicitly.
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::AsRawFd;
            let file = File::open(dir.path().join("data"))?;
            // SAFETY: advice on an open descriptor.
            assert_eq!(
                unsafe { libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_DONTNEED) },
                0
            );
        }
        #[cfg(target_os = "macos")]
        assert!(!crate::region::residency::is_fully_resident(
            mapping,
            region.meta().start().next_multiple_of(16 * 1024),
            16 * 1024
        ));
    }
    // Dirty, unflushed mapped bytes in an otherwise cold chunk must survive its uncached read-modify-write.
    let dirty = 700_001;
    region.write_at(&[9; 50], dirty)?;
    expected[dirty..dirty + 50].fill(9);
    // Scattered values, values touching 16 KiB boundaries, and a dense run of over a MiB, so chunks fill
    // up with values straddling their ends.
    let start = region.meta().start();
    let keys = (0..COUNT).filter(|i| {
        i % 97 == 0 || (start + AT + i * LEN) % 16384 > 16384 - LEN || (10_000..45_000).contains(i)
    });
    write(
        keys.map(|i| (i, [i as u8 ^ 0x5A; LEN]))
            .filter(|(i, _)| !(18_900..18_920).contains(i))
            .collect(),
        &mut expected,
    )?;
    // The pages were just read back, so this pass takes the resident path.
    write(
        (0..COUNT)
            .step_by(7)
            .map(|i| (i, [!(i as u8); LEN]))
            .collect(),
        &mut expected,
    )?;
    #[cfg(target_os = "macos")]
    {
        let _writes = db.inner.writes.read();
        // SAFETY: the barrier keeps the mapping in place for this probe.
        let mapping = unsafe { db.inner.data.mapping() };
        let page = (start + AT + 20_000 * LEN) / 16384 * 16384;
        assert!(crate::region::residency::is_fully_resident(
            mapping, page, 16384
        ));
    }
    assert_eq!(before.create_reader().read_all(), &[1; PAGE_SIZE][..]);
    assert_eq!(after.create_reader().read_all(), &[2; PAGE_SIZE][..]);
    db.flush();
    drop((before, region, after));
    drop(db);

    let db = Database::open(dir.path())?;
    assert_eq!(
        db.get_region("region").unwrap().create_reader().read_all(),
        &expected[..]
    );
    assert_eq!(
        db.get_region("before").unwrap().create_reader().read_all(),
        &[1; PAGE_SIZE][..]
    );
    assert_eq!(
        db.get_region("after").unwrap().create_reader().read_all(),
        &[2; PAGE_SIZE][..]
    );
    Ok(())
}
