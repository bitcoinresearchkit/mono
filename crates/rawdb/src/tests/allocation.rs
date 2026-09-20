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
fn best_fit_reuses_equal_holes_in_address_order() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let low = db.create_region_if_needed("low")?;
    let separator = db.create_region_if_needed("separator")?;
    let high = db.create_region_if_needed("high")?;
    let tail = db.create_region_if_needed("tail")?;
    separator.write(b"separator")?;
    tail.write(b"tail")?;

    // Make the higher hole reusable first. Equal-sized holes still choose the
    // lowest address, and consuming one must preserve the other index entry.
    high.remove()?;
    db.flush()?;
    low.remove()?;
    db.flush()?;
    let first = db.create_region_if_needed("first")?;
    let second = db.create_region_if_needed("second")?;
    assert_eq!(first.meta().start(), 0);
    assert_eq!(second.meta().start(), 2 * PAGE_SIZE);
    assert_eq!(separator.create_reader().read_all(), b"separator");
    assert_eq!(tail.create_reader().read_all(), b"tail");
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

#[test]
fn repeated_unflushed_removals_preserve_data_and_empty_registry() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    for cycle in 0..5 {
        let regions = (0..20)
            .map(|i| db.create_region_if_needed(&format!("cycle_{cycle}_region_{i}")))
            .collect::<Result<Vec<_>>>()?;
        for (i, region) in regions.iter().enumerate() {
            region.write(format!("Cycle {cycle} Region {i}").as_bytes())?;
        }
        for (i, region) in regions.iter().enumerate() {
            assert_eq!(
                region.create_reader().read_all(),
                format!("Cycle {cycle} Region {i}").as_bytes()
            );
        }
        for region in regions {
            region.remove()?;
        }
        assert_eq!(db.regions().len(), 0);
    }
    Ok(())
}

#[test]
fn test_many_small_regions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create 50 small regions
    let mut regions = Vec::new();
    for i in 0..50 {
        let name = format!("region_{}", i);
        let region = db.create_region_if_needed(&name)?;
        let data = format!("Data for region {}", i);
        region.write(data.as_bytes())?;
        regions.push(Some(region));
    }

    // Verify all regions
    for (i, region_opt) in regions.iter().enumerate() {
        let region = region_opt.as_ref().unwrap();
        let reader = region.create_reader();
        let expected = format!("Data for region {}", i);
        assert_eq!(reader.read_all(), expected.as_bytes());
        drop(reader);
    }

    // Remove every other region
    for i in (0..50).step_by(2) {
        let region = regions[i].take().unwrap();
        region.remove()?;
    }

    // Verify remaining regions
    for i in (1..50).step_by(2) {
        let region = regions[i].as_ref().unwrap();
        let reader = region.create_reader();
        let expected = format!("Data for region {}", i);
        assert_eq!(reader.read_all(), expected.as_bytes());
        drop(reader);
    }

    Ok(())
}

#[test]
fn test_hole_coalescing() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create 5 regions
    let mut regions: Vec<_> = (0..5)
        .map(|i| db.create_region_if_needed(&format!("r{}", i)))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(Some)
        .collect();

    // Write small data to each
    for r in &regions {
        r.as_ref().unwrap().write(b"data")?;
    }

    // Remove regions 1, 2, 3 to create adjacent holes
    regions[1].take().unwrap().remove()?;
    regions[2].take().unwrap().remove()?;
    regions[3].take().unwrap().remove()?;
    db.flush()?; // Make holes available and coalesce

    // Check that holes were coalesced
    let layout = db.layout();
    // Should have 1 large hole, not 3 separate ones
    let holes = layout.start_to_hole();
    assert_eq!(holes.len(), 1);

    // The single hole should span all 3 removed regions
    let hole_size = holes.values().next().unwrap();
    assert_eq!(*hole_size, PAGE_SIZE * 3);

    Ok(())
}

#[test]
fn test_complex_fragmentation_scenario() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create pattern: region, region, region, region, region
    let r0 = db.create_region_if_needed("r0")?;
    let r1 = db.create_region_if_needed("r1")?;
    let r2 = db.create_region_if_needed("r2")?;
    let r3 = db.create_region_if_needed("r3")?;
    let r4 = db.create_region_if_needed("r4")?;

    for r in [&r0, &r1, &r2, &r3, &r4] {
        r.write(b"data")?;
    }

    // Remove pattern: keep, remove, keep, remove, keep
    // This creates 2 separate holes
    r1.remove()?;
    r3.remove()?;
    db.flush()?; // Make holes available

    let layout = db.layout();
    assert_eq!(layout.start_to_hole().len(), 2);
    drop(layout);

    // Create a new region - should fill one of the holes
    let r5 = db.create_region_if_needed("r5")?;
    r5.write(b"fills hole")?;

    let layout = db.layout();
    assert_eq!(layout.start_to_hole().len(), 1); // One hole filled, one remains

    Ok(())
}
