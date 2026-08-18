use rawdb::{Database, PAGE_SIZE, Result};
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

/// Helper to create a temporary test database
fn setup_test_db() -> Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db = Database::open(temp_dir.path())?;
    Ok((db, temp_dir))
}

#[test]
fn test_database_creation() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Database should start empty
    assert_eq!(db.regions().index_to_region().len(), 0);
    assert_eq!(db.layout().start_to_region().len(), 0);
    assert_eq!(db.layout().start_to_hole().len(), 0);

    Ok(())
}

#[test]
fn test_create_single_region() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test_region")?;

    // Verify region properties
    let meta = region.meta();
    assert_eq!(meta.start(), 0);
    assert_eq!(meta.len(), 0);
    assert_eq!(meta.reserved(), PAGE_SIZE);
    drop(meta);

    // Verify it's tracked in regions
    let regions = db.regions();
    assert_eq!(regions.index_to_region().len(), 1);
    assert!(regions.get_from_id("test_region").is_some());

    // Verify it's tracked in layout
    let layout = db.layout();
    assert_eq!(layout.start_to_region().len(), 1);
    assert!(layout.start_to_hole().is_empty());

    Ok(())
}

#[test]
fn test_create_region_idempotent() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("test")?;
    let region2 = db.create_region_if_needed("test")?;

    // Should return same region
    assert_eq!(region1.index(), region2.index());
    assert_eq!(db.regions().index_to_region().len(), 1);

    Ok(())
}

#[test]
fn test_reserve_region_capacity_preserves_data() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("first")?;
    region.write(b"preserved")?;
    let blocker = db.create_region_if_needed("blocker")?;
    let initial_start = region.meta().start();

    region.reserve_capacity(PAGE_SIZE * 3 + 1)?;

    let meta = region.meta();
    assert_ne!(meta.start(), initial_start);
    assert_eq!(meta.len(), b"preserved".len());
    assert_eq!(meta.reserved(), PAGE_SIZE * 4);
    drop(meta);
    assert_eq!(region.create_reader().read_all(), b"preserved");
    assert_eq!(blocker.meta().start(), PAGE_SIZE);

    db.flush()?;
    drop(blocker);
    drop(region);
    drop(db);

    let reopened = Database::open(_temp.path())?;
    let region = reopened.get_region("first").unwrap();
    assert_eq!(region.meta().reserved(), PAGE_SIZE * 4);
    assert_eq!(region.create_reader().read_all(), b"preserved");

    Ok(())
}

#[test]
fn test_write_to_region_within_reserved() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    let data = b"Hello, World!";

    region.write(data)?;

    // Verify data was written
    let meta = region.meta();
    assert_eq!(meta.len(), data.len());
    assert_eq!(meta.reserved(), PAGE_SIZE);
    let start = meta.start();
    drop(meta);

    let mmap = db.mmap();
    assert_eq!(&mmap[start..start + data.len()], data);

    Ok(())
}

#[test]
fn test_write_append() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    region.write(b"Hello")?;
    region.write(b", World!")?;

    let meta = region.meta();
    assert_eq!(meta.len(), 13);
    let start = meta.start();
    drop(meta);

    let mmap = db.mmap();
    assert_eq!(&mmap[start..(start + 13)], b"Hello, World!");

    Ok(())
}

#[test]
fn test_write_at_position() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    region.write(b"Hello, World!")?;
    region.write_at(b"Rust!", 7)?;

    let meta = region.meta();
    let start = meta.start();
    drop(meta);

    let mmap = db.mmap();
    assert_eq!(&mmap[start..(start + 13)], b"Hello, Rust!!");

    Ok(())
}

#[test]
fn test_write_exceeds_reserved() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    // Write more than PAGE_SIZE to trigger expansion
    let large_data = vec![1u8; PAGE_SIZE + 100];
    region.write(&large_data)?;

    let meta = region.meta();
    assert_eq!(meta.len(), large_data.len());
    assert!(meta.reserved() >= PAGE_SIZE * 2);

    Ok(())
}

#[test]
fn test_truncate_region() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    region.write(b"Hello, World!")?;

    let meta_before = region.meta();
    assert_eq!(meta_before.len(), 13);
    drop(meta_before);

    region.truncate(5)?;

    let meta_after = region.meta();
    assert_eq!(meta_after.len(), 5);

    Ok(())
}

#[test]
fn test_truncate_errors() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Hello")?;

    // Truncating beyond length should error
    let result = region.truncate(10);
    assert!(result.is_err());

    // Truncating to same length should be OK
    let result = region.truncate(5);
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_remove_region() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    let index = region.index();

    region.write(b"Hello")?;

    // Remove region
    region.remove()?;

    // Verify removal
    let regions = db.regions();
    assert!(regions.get_from_id("test").is_none());
    assert!(regions.get_from_index(index).is_none());

    db.flush()?; // Make hole available

    // Layout should have a hole now
    let layout = db.layout();
    assert_eq!(layout.start_to_region().len(), 0);
    assert_eq!(layout.start_to_hole().len(), 1);

    Ok(())
}

#[test]
fn test_multiple_regions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("region1")?;
    let region2 = db.create_region_if_needed("region2")?;
    let region3 = db.create_region_if_needed("region3")?;

    // Write different data to each
    region1.write(b"First")?;
    region2.write(b"Second")?;
    region3.write(b"Third")?;

    // Verify all exist
    assert_eq!(db.regions().index_to_region().len(), 3);
    assert_eq!(db.layout().start_to_region().len(), 3);

    // Verify data integrity
    let mmap = db.mmap();

    let meta1 = region1.meta();
    assert_eq!(&mmap[meta1.start()..(meta1.start() + 5)], b"First");
    drop(meta1);

    let meta2 = region2.meta();
    assert_eq!(&mmap[meta2.start()..(meta2.start() + 6)], b"Second");
    drop(meta2);

    let meta3 = region3.meta();
    assert_eq!(&mmap[meta3.start()..(meta3.start() + 5)], b"Third");

    Ok(())
}

#[test]
fn test_region_reuse_after_removal() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("region1")?;
    let _region2 = db.create_region_if_needed("region2")?;
    let index1 = region1.index();

    // Remove first region
    region1.remove()?;

    // Create a new region - should reuse the slot
    let region3 = db.create_region_if_needed("region3")?;
    assert_eq!(region3.index(), index1);

    // Verify only 2 regions exist
    let regions = db.regions();
    assert_eq!(regions.id_to_index().len(), 2);
    assert!(regions.get_from_id("region1").is_none());
    assert!(regions.get_from_id("region2").is_some());
    assert!(regions.get_from_id("region3").is_some());

    Ok(())
}

#[test]
fn test_hole_filling() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let _region1 = db.create_region_if_needed("region1")?;
    let region2 = db.create_region_if_needed("region2")?;
    let _region3 = db.create_region_if_needed("region3")?;

    // Remove middle region to create a hole
    region2.remove()?;
    db.flush()?; // Make hole available for reuse

    let layout = db.layout();
    assert_eq!(layout.start_to_hole().len(), 1);
    drop(layout);

    // Create new region - should fill the hole
    let _region4 = db.create_region_if_needed("region4")?;

    let layout = db.layout();
    // Hole should be gone since new region takes PAGE_SIZE which fills it exactly
    assert_eq!(layout.start_to_hole().len(), 0);

    Ok(())
}

#[test]
fn test_persistence() -> Result<()> {
    let temp = TempDir::new()?;
    let path = temp.path();
    dbg!(&path);

    // Create and populate database
    {
        let db = Database::open(path)?;
        let region = db.create_region_if_needed("persistent")?;
        region.write(b"Persisted data")?;
        db.flush()?;
    }

    // Reopen and verify
    {
        let db = Database::open(path)?;
        let regions = db.regions();
        let region = regions
            .get_from_id("persistent")
            .expect("Region should persist");

        let meta = region.meta();
        assert_eq!(meta.len(), 14);
        let start = meta.start();
        drop(meta);

        let mmap = db.mmap();
        assert_eq!(&mmap[start..(start + 14)], b"Persisted data");
    }

    Ok(())
}

#[test]
fn test_empty_region_persistence() -> Result<()> {
    let temp = TempDir::new()?;

    {
        let db = Database::open(temp.path())?;
        let region = db.create_region_if_needed("empty")?;
        assert!(region.flush()?);
        assert!(!region.flush()?);
    }

    let db = Database::open(temp.path())?;
    let regions = db.regions();
    let region = regions
        .get_from_id("empty")
        .expect("empty region should persist");
    assert_eq!(region.meta().len(), 0);

    Ok(())
}

#[test]
fn test_reader() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Hello, World!")?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"Hello, World!");
    assert_eq!(reader.read(0, 5), b"Hello");
    assert_eq!(reader.read(7, 5), b"World");

    Ok(())
}

#[test]
fn test_retain_regions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let _ = db.create_region_if_needed("keep1")?;
    let _ = db.create_region_if_needed("remove1")?;
    let _ = db.create_region_if_needed("keep2")?;
    let _ = db.create_region_if_needed("remove2")?;

    let mut keep_set = std::collections::HashSet::new();
    keep_set.insert("keep1".to_string());
    keep_set.insert("keep2".to_string());

    db.retain_regions(keep_set)?;

    let regions = db.regions();
    assert_eq!(regions.id_to_index().len(), 2);
    assert!(regions.get_from_id("keep1").is_some());
    assert!(regions.get_from_id("keep2").is_some());
    assert!(regions.get_from_id("remove1").is_none());
    assert!(regions.get_from_id("remove2").is_none());

    Ok(())
}

#[test]
fn test_region_defragmentation() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("region1")?;
    let region2 = db.create_region_if_needed("region2")?;

    dbg!(0);

    // Write small data first
    region1.write(b"small")?;

    dbg!(1);
    // Write large data to region1 - should move it to end
    let large_data = vec![1u8; PAGE_SIZE * 2];
    region1.write(&large_data)?;
    db.flush()?; // Make hole available

    dbg!(2);
    // region1 should have moved, leaving a hole
    let layout = db.layout();
    assert!(layout.start_to_hole().len() == 1);

    dbg!(3);
    // region2 should still be at its original position
    let meta2 = region2.meta();
    assert_eq!(meta2.start(), PAGE_SIZE);
    dbg!(4);

    Ok(())
}

#[test]
fn test_concurrent_region_creation() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Arc::new(Database::open(temp.path())?);

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let db = Arc::clone(&db);
            thread::spawn(move || {
                let region_name = format!("region_{}", i);
                db.create_region_if_needed(&region_name)
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        let _ = handle.join().unwrap()?;
    }

    // Verify all regions created
    let regions = db.regions();
    assert_eq!(regions.id_to_index().len(), 10);

    Ok(())
}

#[test]
fn test_set_min_regions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    db.set_min_regions(100)?;

    // File should be large enough for 100 regions
    let file_len = db.file_len();
    assert!(file_len >= 100 * PAGE_SIZE);

    Ok(())
}

#[test]
fn test_large_write() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("large")?;

    // Write 1MB of data
    let large_data = vec![42u8; 1024 * 1024];
    region.write(&large_data)?;

    let meta = region.meta();
    assert_eq!(meta.len(), large_data.len());
    let start = meta.start();
    drop(meta);

    // Verify data
    let mmap = db.mmap();
    assert_eq!(&mmap[start..(start + large_data.len())], &large_data[..]);

    Ok(())
}

#[test]
fn test_truncate_write() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    region.write(b"Hello, World!")?;

    let meta_before = region.meta();
    assert_eq!(meta_before.len(), 13);
    drop(meta_before);

    // Truncate write - should set length to exactly the written data
    region.truncate_write(7, b"Rust")?;

    let meta_after = region.meta();
    assert_eq!(meta_after.len(), 11); // 7 + 4
    let start = meta_after.start();
    drop(meta_after);

    let mmap = db.mmap();
    assert_eq!(&mmap[start..(start + 11)], b"Hello, Rust");

    Ok(())
}

#[test]
fn test_punch_holes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    // Allocate the pages, then fill them with zeroes. Allocated zero-filled
    // pages must still be reclaimed; their contents do not identify holes.
    let mut large_data = vec![1u8; PAGE_SIZE * 4];
    region.write(&large_data)?;
    large_data.fill(0);
    region.write_at(&large_data, 0)?;
    db.flush()?;
    let allocated_before = db.disk_usage()?.bytes();

    region.truncate(100)?;
    db.compact()?;
    let allocated_after = db.disk_usage()?.bytes();

    let meta = region.meta();
    assert_eq!(meta.len(), 100);
    assert!(allocated_after < allocated_before);

    Ok(())
}

#[test]
fn test_opened_region_tail_is_reclaimed() -> Result<()> {
    let temp = TempDir::new()?;

    let allocated_before = {
        let db = Database::open(temp.path())?;
        let region = db.create_region_if_needed("test")?;
        let mut data = vec![1u8; PAGE_SIZE * 4];
        region.write(&data)?;
        data.fill(0);
        region.write_at(&data, 0)?;
        region.truncate(100)?;
        db.flush()?;
        db.disk_usage()?.bytes()
    };

    let db = Database::open(temp.path())?;
    db.compact()?;
    let allocated_after = db.disk_usage()?.bytes();

    assert!(allocated_after < allocated_before);

    Ok(())
}

#[test]
fn test_zero_filled_removed_region_is_reclaimed() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let removed = db.create_region_if_needed("removed")?;
    let mut data = vec![1u8; PAGE_SIZE * 16];
    removed.write(&data)?;
    data.fill(0);
    removed.write_at(&data, 0)?;

    let retained = db.create_region_if_needed("retained")?;
    retained.write(b"retained")?;
    db.flush()?;
    let allocated_before = db.disk_usage()?.bytes();

    removed.remove()?;
    db.compact()?;
    let allocated_after = db.disk_usage()?.bytes();

    assert!(allocated_after < allocated_before);
    assert_eq!(retained.create_reader().read_all(), b"retained");

    Ok(())
}

#[test]
fn test_write_at_invalid_position() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Hello")?;

    // Writing beyond length should fail
    let result = region.write_at(b"World", 10);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_empty_region_operations() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("empty")?;

    // Reading empty region
    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"");
    drop(reader);

    // Truncating empty region to 0 should work
    region.truncate(0)?;

    let meta = region.meta();
    assert_eq!(meta.len(), 0);

    Ok(())
}

#[test]
fn test_region_metadata_updates() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;

    // Initial state
    {
        let meta = region.meta();
        assert_eq!(meta.start(), 0);
        assert_eq!(meta.len(), 0);
        assert_eq!(meta.reserved(), PAGE_SIZE);
    }

    // After first write
    region.write(b"Hello")?;
    {
        let meta = region.meta();
        assert_eq!(meta.len(), 5);
        assert_eq!(meta.reserved(), PAGE_SIZE);
    }

    // After expansion
    let large = vec![1u8; PAGE_SIZE * 3];
    region.write(&large)?;
    {
        let meta = region.meta();
        assert_eq!(meta.len(), 5 + large.len());
        assert!(meta.reserved() >= PAGE_SIZE * 4);
    }

    Ok(())
}

// ============================================================================
// Complex Integration Tests
// ============================================================================

#[test]
fn test_complex_region_lifecycle() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create multiple regions
    let r1 = db.create_region_if_needed("region1")?;
    let r2 = db.create_region_if_needed("region2")?;
    let r3 = db.create_region_if_needed("region3")?;

    // Write to all regions
    r1.write(b"Data for region 1")?;
    r2.write(b"Data for region 2")?;
    r3.write(b"Data for region 3")?;

    // Remove middle region
    r2.remove()?;
    db.flush()?; // Make hole available

    // Verify hole exists
    {
        let layout = db.layout();
        assert_eq!(layout.start_to_hole().len(), 1);
    }

    // Create new region that should reuse the hole
    let r4 = db.create_region_if_needed("region4")?;
    r4.write(b"Fills the hole")?;

    // Verify hole was filled
    {
        let layout = db.layout();
        assert_eq!(layout.start_to_hole().len(), 0);
    }

    // Write large data to trigger region movement (overwrite from start)
    let large = vec![42u8; PAGE_SIZE * 3];
    r4.write_at(&large, 0)?;
    db.flush()?; // Make hole available

    // Verify r4 moved and created a hole
    {
        let layout = db.layout();
        assert!(!layout.start_to_hole().is_empty());
    }

    // Verify all data is still correct
    {
        let reader = r1.create_reader();
        assert_eq!(reader.read_all(), b"Data for region 1");
        drop(reader);
    }

    {
        let reader = r3.create_reader();
        assert_eq!(reader.read_all(), b"Data for region 3");
        drop(reader);
    }

    {
        let reader = r4.create_reader();
        assert_eq!(reader.read_all(), &large[..]);
        drop(reader);
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
fn test_interleaved_operations() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let r1 = db.create_region_if_needed("r1")?;
    let r2 = db.create_region_if_needed("r2")?;
    let r3 = db.create_region_if_needed("r3")?;

    // Interleave writes
    r1.write(b"Start1")?;
    r2.write(b"Start2")?;
    r3.write(b"Start3")?;

    r1.write(b" More1")?;
    r2.write(b" More2")?;

    // Truncate one
    r3.truncate(3)?;

    // Continue writing
    r1.write(b" End1")?;
    r2.write_at(b"X", 0)?;

    // Verify results
    {
        let reader = r1.create_reader();
        assert_eq!(reader.read_all(), b"Start1 More1 End1");
        drop(reader);
    }

    {
        let reader = r2.create_reader();
        assert_eq!(reader.read_all(), b"Xtart2 More2");
        drop(reader);
    }

    {
        let meta = r3.meta();
        assert_eq!(meta.len(), 3);
    }

    Ok(())
}

#[test]
fn test_persistence_with_holes() -> Result<()> {
    let temp = TempDir::new()?;
    let path = temp.path();

    // Create database with regions and holes
    {
        let db = Database::open(path)?;

        let r1 = db.create_region_if_needed("keep1")?;
        let r2 = db.create_region_if_needed("remove")?;
        let r3 = db.create_region_if_needed("keep2")?;

        r1.write(b"Keep this 1")?;
        r2.write(b"Remove this")?;
        r3.write(b"Keep this 2")?;

        r2.remove()?;
        db.flush()?;
    }

    // Reopen and verify
    {
        let db = Database::open(path)?;

        let regions = db.regions();
        assert!(regions.get_from_id("keep1").is_some());
        assert!(regions.get_from_id("remove").is_none());
        assert!(regions.get_from_id("keep2").is_some());

        let r1 = regions.get_from_id("keep1").unwrap();
        let r3 = regions.get_from_id("keep2").unwrap();

        let reader1 = r1.create_reader();
        assert_eq!(reader1.read_all(), b"Keep this 1");
        drop(reader1);

        let reader3 = r3.create_reader();
        assert_eq!(reader3.read_all(), b"Keep this 2");
        drop(reader3);

        // Verify hole still exists
        let layout = db.layout();
        assert!(!layout.start_to_hole().is_empty());
    }

    Ok(())
}

#[test]
fn test_region_growth_patterns() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("growing")?;

    // Grow gradually
    for i in 0..10 {
        let data = vec![i as u8; 1000];
        region.write(&data)?;
    }

    let meta = region.meta();
    assert_eq!(meta.len(), 10_000);

    // Verify all data
    let reader = region.create_reader();
    let all_data = reader.read_all();
    for i in 0..10 {
        let chunk = &all_data[i * 1000..(i + 1) * 1000];
        assert!(chunk.iter().all(|&b| b == i as u8));
    }

    Ok(())
}

#[test]
fn test_write_at_boundary_conditions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("boundary")?;

    // Write at start
    region.write(b"0123456789")?;

    // Write at exact length boundary
    region.write_at(b"ABC", 10)?;

    // Write at position 0
    region.write_at(b"X", 0)?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"X123456789ABC");

    Ok(())
}

#[test]
fn test_multiple_flushes() -> Result<()> {
    let temp = TempDir::new()?;
    let path = temp.path();

    {
        let db = Database::open(path)?;
        let r = db.create_region_if_needed("test")?;

        r.write(b"Version 1")?;
        db.flush()?;

        r.write(b" Version 2")?;
        db.flush()?;

        r.write(b" Version 3")?;
        db.flush()?;
    }

    {
        let db = Database::open(path)?;
        let regions = db.regions();
        let r = regions.get_from_id("test").unwrap();

        let reader = r.create_reader();
        assert_eq!(reader.read_all(), b"Version 1 Version 2 Version 3");
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
fn test_stress_region_creation_and_removal() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create and remove regions in a cycle
    for cycle in 0..5 {
        // Create 20 regions
        let regions: Vec<_> = (0..20)
            .map(|i| {
                let name = format!("cycle_{}_region_{}", cycle, i);
                db.create_region_if_needed(&name)
            })
            .collect::<Result<Vec<_>>>()?;

        // Write to each
        for (i, r) in regions.iter().enumerate() {
            let data = format!("Cycle {} Region {}", cycle, i);
            r.write(data.as_bytes())?;
        }

        // Verify
        for (i, r) in regions.iter().enumerate() {
            let reader = r.create_reader();
            let expected = format!("Cycle {} Region {}", cycle, i);
            assert_eq!(reader.read_all(), expected.as_bytes());
            drop(reader);
        }

        // Remove all
        for r in regions {
            r.remove()?;
        }

        // Verify all gone
        let reg = db.regions();
        assert_eq!(reg.id_to_index().len(), 0);
    }

    Ok(())
}

#[test]
fn test_mixed_size_writes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("mixed")?;

    // Write various sizes
    region.write(b"tiny")?;
    region.write(&[1u8; 100])?;
    region.write(&[2u8; 1000])?;
    region.write(&[3u8; 10000])?;

    let meta = region.meta();
    assert_eq!(meta.len(), 4 + 100 + 1000 + 10000);

    // Verify each section
    let reader = region.create_reader();
    assert_eq!(&reader.read(0, 4), b"tiny");
    assert!(reader.read(4, 100).iter().all(|&b| b == 1));
    assert!(reader.read(104, 1000).iter().all(|&b| b == 2));
    assert!(reader.read(1104, 10000).iter().all(|&b| b == 3));

    Ok(())
}

// ============================================================================
// Concurrent Operations Tests
// ============================================================================

#[test]
fn test_concurrent_writes_to_different_regions() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Arc::new(Database::open(temp.path())?);

    // Create regions upfront
    let regions: Vec<_> = (0..10)
        .map(|i| db.create_region_if_needed(&format!("region_{}", i)))
        .collect::<Result<Vec<_>>>()?;

    // Write to different regions concurrently
    let handles: Vec<_> = regions
        .into_iter()
        .enumerate()
        .map(|(i, region)| {
            thread::spawn(move || {
                let data = vec![i as u8; 1000];
                region.write(&data)
            })
        })
        .collect();

    // Wait for all writes
    for handle in handles {
        handle.join().unwrap()?;
    }

    // Verify all data
    for i in 0..10 {
        let regions = db.regions();
        let region = regions.get_from_id(&format!("region_{}", i)).unwrap();
        let reader = region.create_reader();
        let data = reader.read_all();
        assert_eq!(data.len(), 1000);
        assert!(data.iter().all(|&b| b == i as u8));
    }

    Ok(())
}

#[test]
fn test_concurrent_reads() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let db = Arc::new(db);

    let region = db.create_region_if_needed("shared")?;
    let data = b"Shared data for concurrent reads";
    region.write(data)?;

    // Multiple threads reading simultaneously
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let db = Arc::clone(&db);
            thread::spawn(move || {
                let regions = db.regions();
                let region = regions.get_from_id("shared").unwrap();
                let reader = region.create_reader();
                assert_eq!(reader.read_all(), b"Shared data for concurrent reads");
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

// ============================================================================
// Reader Edge Cases
// ============================================================================

#[test]
fn test_reader_prefixed() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"0123456789ABCDEF")?;

    let reader = region.create_reader();

    // Test prefixed reads
    let prefixed = reader.prefixed(5);
    assert!(prefixed.starts_with(b"56789ABCDEF"));

    let prefixed_at_start = reader.prefixed(0);
    assert!(prefixed_at_start.starts_with(b"0123456789ABCDEF"));

    Ok(())
}

#[test]
fn test_reader_unchecked_read() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Hello World")?;

    let reader = region.create_reader();

    // Unchecked read within bounds should work
    let data = reader.unchecked_read(0, 5);
    assert_eq!(data, b"Hello");

    let data = reader.unchecked_read(6, 5);
    assert_eq!(data, b"World");

    Ok(())
}

#[test]
#[should_panic]
fn test_reader_bounds_check() {
    let (db, _temp) = setup_test_db().unwrap();

    let region = db.create_region_if_needed("test").unwrap();
    region.write(b"Short").unwrap();

    let reader = region.create_reader();

    // This should panic due to bounds check
    let _ = reader.read(0, 100);
}

// Test that Reader is self-contained and safe to use after dropping original references
#[test]
fn test_reader_outlives_region_variable() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let reader = {
        let region = db.create_region_if_needed("test")?;
        region.write(b"Hello from dropped region")?;
        region.create_reader()
        // region is dropped here, but reader should still be valid
    };

    // Reader should still work because it holds its own Arc references
    assert_eq!(reader.read_all(), b"Hello from dropped region");
    assert_eq!(reader.read(0, 5), b"Hello");

    Ok(())
}

#[test]
fn test_reader_outlives_database_variable() -> Result<()> {
    let (_temp, reader) = {
        let (db, temp) = setup_test_db()?;
        let region = db.create_region_if_needed("test")?;
        region.write(b"Persisted data")?;
        let reader = region.create_reader();
        // Both db and region go out of scope, but reader holds Arc clones
        (temp, reader)
    };

    // Reader should still work
    assert_eq!(reader.read_all(), b"Persisted data");

    Ok(())
}

#[test]
fn test_reader_can_be_stored_in_struct() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    struct DataHolder {
        reader: rawdb::Reader,
    }

    let region = db.create_region_if_needed("test")?;
    region.write(b"Stored in struct")?;

    let holder = DataHolder {
        reader: region.create_reader(),
    };

    // Can use reader through struct
    assert_eq!(holder.reader.read_all(), b"Stored in struct");

    Ok(())
}

#[test]
fn test_multiple_readers_from_same_region() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Shared data")?;

    // Create multiple readers
    let reader1 = region.create_reader();
    let reader2 = region.create_reader();
    let reader3 = region.create_reader();

    // All should read the same data
    assert_eq!(reader1.read_all(), b"Shared data");
    assert_eq!(reader2.read_all(), b"Shared data");
    assert_eq!(reader3.read_all(), b"Shared data");

    // Drop in different order
    drop(reader2);
    assert_eq!(reader1.read_all(), b"Shared data");
    assert_eq!(reader3.read_all(), b"Shared data");

    Ok(())
}

// ============================================================================
// Extreme Cases
// ============================================================================

#[test]
fn test_very_long_region_names() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create regions with very long names
    let long_name = "a".repeat(1000);
    let region = db.create_region_if_needed(&long_name)?;
    region.write(b"data")?;

    // Verify it persists
    db.flush()?;

    let regions = db.regions();
    let retrieved = regions.get_from_id(&long_name);
    assert!(retrieved.is_some());

    Ok(())
}

#[test]
fn test_zero_byte_writes() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("empty_writes")?;

    // Write zero bytes
    region.write(b"")?;

    let meta = region.meta();
    assert_eq!(meta.len(), 0);
    drop(meta);

    // Write some data, then write zero bytes again
    region.write(b"Hello")?;
    region.write(b"")?;

    let meta = region.meta();
    assert_eq!(meta.len(), 5);

    Ok(())
}

#[test]
fn test_alternating_write_and_truncate() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("oscillating")?;

    for cycle in 0..10 {
        // Grow
        let data = vec![cycle as u8; 1000];
        region.write(&data)?;

        let meta = region.meta();
        let expected_len = if cycle == 0 { 1000 } else { 100 + 1000 };
        assert_eq!(meta.len(), expected_len);
        drop(meta);

        // Shrink
        region.truncate(100)?;

        let meta = region.meta();
        assert_eq!(meta.len(), 100);
        drop(meta);
    }

    Ok(())
}

// ============================================================================
// Persistence Edge Cases
// ============================================================================

#[test]
fn test_retain_regions_edge_cases() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create some regions
    let _ = db.create_region_if_needed("keep1")?;
    let _ = db.create_region_if_needed("keep2")?;
    let _ = db.create_region_if_needed("remove1")?;

    // Retain with empty set - should remove all
    let empty_set = std::collections::HashSet::new();
    db.retain_regions(empty_set)?;

    let regions = db.regions();
    assert_eq!(regions.id_to_index().len(), 0);

    Ok(())
}

// ============================================================================
// Layout and Hole Management
// ============================================================================

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

#[test]
fn test_set_min_len_preallocate() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Preallocate large file
    let large_size = PAGE_SIZE * 1000;
    db.set_min_len(large_size)?;

    let file_len = db.file_len();
    assert!(file_len >= large_size);

    // Should still be able to write
    let region = db.create_region_if_needed("test")?;
    region.write(b"After preallocation")?;

    Ok(())
}

// ============================================================================
// Data Integrity Tests
// ============================================================================

#[test]
fn test_partial_overwrites_data_integrity() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("partial")?;

    // Write initial pattern
    let initial = b"AAAAAAAAAA";
    region.write(initial)?;

    // Overwrite middle
    region.write_at(b"BBB", 3)?;

    // Overwrite start
    region.write_at(b"CC", 0)?;

    // Overwrite end
    region.write_at(b"DD", 8)?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"CCABBBAADD");

    Ok(())
}

#[test]
fn test_write_at_exact_reserved_boundary() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("boundary")?;

    // Fill exactly to PAGE_SIZE
    let data = vec![42u8; PAGE_SIZE];
    region.write(&data)?;

    let meta = region.meta();
    assert_eq!(meta.len(), PAGE_SIZE);
    assert_eq!(meta.reserved(), PAGE_SIZE);
    drop(meta);

    // Writing one more byte should trigger expansion
    region.write(b"X")?;

    let meta = region.meta();
    assert_eq!(meta.len(), PAGE_SIZE + 1);
    assert!(meta.reserved() > PAGE_SIZE);

    Ok(())
}

// ============================================================================
// Comprehensive Integration Test
// ============================================================================

#[test]
fn test_comprehensive_db_operations() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("region1")?;

    {
        let layout = db.layout();

        assert!(layout.start_to_region().len() == 1);

        assert!(layout.start_to_hole().is_empty());

        let regions = db.regions();

        assert!(
            regions
                .get_from_id("region1")
                .is_some_and(|r| r.ptr_eq(&region1))
        );

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 0);
        assert!(region1_meta.reserved() == PAGE_SIZE);
    }

    region1.write(&[0, 1, 2, 3, 4])?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 5);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(db.mmap()[0..10] == [0, 1, 2, 3, 4, 0, 0, 0, 0, 0]);
    }

    region1.write(&[5, 6, 7, 8, 9])?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 10);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(db.mmap()[0..10] == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    region1.write_at(&[1, 2], 0)?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 10);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(db.mmap()[0..10] == [1, 2, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    region1.write_at(&[10, 11, 12, 13, 14, 15, 16, 17, 18], 4)?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 13);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(
            db.mmap()[0..20]
                == [
                    1, 2, 2, 3, 10, 11, 12, 13, 14, 15, 16, 17, 18, 0, 0, 0, 0, 0, 0, 0
                ]
        );
    }

    region1.write_at(&[0, 0, 0, 0, 0, 1], 13)?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 19);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(
            db.mmap()[0..20]
                == [
                    1, 2, 2, 3, 10, 11, 12, 13, 14, 15, 16, 17, 18, 0, 0, 0, 0, 0, 1, 0
                ]
        );
    }

    region1.write_at(&[1; 8000], 0)?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == PAGE_SIZE * 2);
        assert!(db.mmap()[0..8000] == [1; 8000]);
        assert!(db.mmap()[8000..8001] == [0]);
    }

    db.flush()?;

    region1.truncate(10)?;
    db.compact()?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 10);
        assert!(region1_meta.reserved() == PAGE_SIZE * 2);

        // We only punch a hole in whole pages (4096 bytes)
        // Thus the last byte of the page where the is still data wasn't overwritten when truncating
        // And the first byte of the punched page was set to 0
        assert!(db.mmap()[4095..=4096] == [1, 0]);
    }

    db.flush()?;

    region1.truncate(10)?;
    db.compact()?;

    {
        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 10);
        assert!(region1_meta.reserved() == PAGE_SIZE * 2);
        // We only punch a hole in whole pages (4096 bytes)
        // Thus the last byte of the page where the is still data wasn't overwritten when truncating
        // And the first byte of the punched page was set to 0
        assert!(db.mmap()[4095..=4096] == [1, 0]);
    }

    db.flush()?;

    region1.remove()?;
    db.compact()?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 1);
        assert!(index_to_region[0].is_none());
        assert!(regions.id_to_index().is_empty());

        let layout = db.layout();
        assert!(layout.start_to_region().is_empty());
        assert!(layout.start_to_hole().len() == 1);
    }

    let region1 = db.create_region_if_needed("region1")?;
    let region2 = db.create_region_if_needed("region2")?;
    let region3 = db.create_region_if_needed("region3")?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 0);
        assert!(region1_meta.reserved() == PAGE_SIZE);

        let region2_meta = region2.meta();
        assert!(region2_meta.start() == PAGE_SIZE);
        assert!(region2_meta.len() == 0);
        assert!(region2_meta.reserved() == PAGE_SIZE);

        let region3_meta = region3.meta();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.len() == 0);
        assert!(region3_meta.reserved() == PAGE_SIZE);

        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 3);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 3);

        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(start_to_index.get(&PAGE_SIZE).unwrap().ptr_eq(&region2));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 2))
                .unwrap()
                .ptr_eq(&region3)
        );
        assert!(layout.start_to_hole().is_empty());
    }

    region2.remove()?;
    db.compact()?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 0);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(index_to_region.get(1).is_some_and(|opt| opt.is_none()));

        let region3_meta = region3.meta();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.len() == 0);
        assert!(region3_meta.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2").is_none());
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 2))
                .unwrap()
                .ptr_eq(&region3)
        );
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.len() == 1);
        assert!(start_to_hole.get(&PAGE_SIZE) == Some(&PAGE_SIZE));
    }

    let region2 = db.create_region_if_needed("region2")?;
    let region2_i = region2.index();
    assert!(region2_i == 1);

    region2.remove()?;
    db.compact()?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 0);
        assert!(region1_meta.reserved() == PAGE_SIZE);
        assert!(
            index_to_region
                .get(region2_i)
                .is_some_and(|opt| opt.is_none())
        );

        let region3_meta = region3.meta();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.len() == 0);
        assert!(region3_meta.reserved() == PAGE_SIZE);

        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2").is_none());
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 2))
                .unwrap()
                .ptr_eq(&region3)
        );

        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.len() == 1);
        assert!(start_to_hole.get(&PAGE_SIZE) == Some(&PAGE_SIZE));
    }

    region1.write_at(&[1; 8000], 0)?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == 2 * PAGE_SIZE);
        assert!(
            index_to_region
                .get(region2_i)
                .is_some_and(|opt| opt.is_none())
        );

        let region3_meta = region3.meta();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.len() == 0);
        assert!(region3_meta.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2").is_none());
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 2))
                .unwrap()
                .ptr_eq(&region3)
        );
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.is_empty());
    }

    let region2 = db.create_region_if_needed("region2")?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == 2 * PAGE_SIZE);

        let region2_meta = region2.meta();
        assert!(region2_meta.start() == PAGE_SIZE * 3);
        assert!(region2_meta.len() == 0);
        assert!(region2_meta.reserved() == PAGE_SIZE);

        let region3_meta = region3.meta();
        assert!(region3_meta.start() == PAGE_SIZE * 2);
        assert!(region3_meta.len() == 0);
        assert!(region3_meta.reserved() == PAGE_SIZE);
        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 3);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3") == Some(&2));

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 3);
        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 2))
                .unwrap()
                .ptr_eq(&region3)
        );
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 3))
                .unwrap()
                .ptr_eq(&region2)
        );
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.is_empty());
    }

    region3.remove()?;
    db.compact()?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == 0);
        assert!(region1_meta.len() == 8000);
        assert!(region1_meta.reserved() == 2 * PAGE_SIZE);

        let region2_meta = region2.meta();
        assert!(region2_meta.start() == PAGE_SIZE * 3);
        assert!(region2_meta.len() == 0);
        assert!(region2_meta.reserved() == PAGE_SIZE);

        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3").is_none());

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(start_to_index.get(&0).unwrap().ptr_eq(&region1));
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 3))
                .unwrap()
                .ptr_eq(&region2)
        );
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.get(&(PAGE_SIZE * 2)) == Some(&PAGE_SIZE));
    }

    region1.write(&[1; 8000])?;
    db.compact()?;

    {
        let regions = db.regions();
        let index_to_region = regions.index_to_region();
        assert!(index_to_region.len() == 3);

        let region1_meta = region1.meta();
        assert!(region1_meta.start() == PAGE_SIZE * 4);
        assert!(region1_meta.len() == 16_000);
        assert!(region1_meta.reserved() == 4 * PAGE_SIZE);

        let region2_meta = region2.meta();
        assert!(region2_meta.start() == PAGE_SIZE * 3);
        assert!(region2_meta.len() == 0);
        assert!(region2_meta.reserved() == PAGE_SIZE);

        let id_to_index = regions.id_to_index();
        assert!(id_to_index.len() == 2);
        assert!(id_to_index.get("region1") == Some(&0));
        assert!(id_to_index.get("region2") == Some(&1));
        assert!(id_to_index.get("region3").is_none());

        let layout = db.layout();
        let start_to_index = layout.start_to_region();
        assert!(start_to_index.len() == 2);
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 4))
                .unwrap()
                .ptr_eq(&region1)
        );
        assert!(
            start_to_index
                .get(&(PAGE_SIZE * 3))
                .unwrap()
                .ptr_eq(&region2)
        );
        let start_to_hole = layout.start_to_hole();
        assert!(start_to_hole.get(&0) == Some(&(PAGE_SIZE * 3)));
    }

    region2.write(&[1; 6000])?;

    let region4 = db.create_region_if_needed("region4")?;
    region2.remove()?;
    region4.remove()?;

    Ok(())
}

// ============================================================================
// Region Rename Tests
// ============================================================================

#[test]
fn test_basic_region_rename() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("old_name")?;
    region.write(b"Test data")?;

    // Verify old name exists
    {
        let regions = db.regions();
        assert!(regions.get_from_id("old_name").is_some());
        assert!(regions.get_from_id("new_name").is_none());
    }

    // Rename the region
    region.rename("new_name")?;

    // Verify new name exists and old name doesn't
    {
        let regions = db.regions();
        assert!(regions.get_from_id("old_name").is_none());
        assert!(regions.get_from_id("new_name").is_some());
    }

    // Verify data is still intact
    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"Test data");
    drop(reader);

    // Verify metadata was updated
    let meta = region.meta();
    assert_eq!(meta.id(), "new_name");

    Ok(())
}

#[test]
fn test_rename_with_persistence() -> Result<()> {
    let temp = TempDir::new()?;
    let path = temp.path();

    // Create and rename region
    {
        let db = Database::open(path)?;
        let region = db.create_region_if_needed("original")?;
        region.write(b"Persistent data")?;
        region.rename("renamed")?;
        db.flush()?;
    }

    // Reopen and verify rename persisted
    {
        let db = Database::open(path)?;
        let regions = db.regions();

        assert!(regions.get_from_id("original").is_none());
        let renamed = regions.get_from_id("renamed");
        assert!(renamed.is_some());

        let reader = renamed.unwrap().create_reader();
        assert_eq!(reader.read_all(), b"Persistent data");
    }

    Ok(())
}

#[test]
fn test_rename_to_existing_name_fails() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region1 = db.create_region_if_needed("region1")?;
    let _region2 = db.create_region_if_needed("region2")?;

    // Trying to rename region1 to region2 should fail
    let result = region1.rename("region2");
    assert!(result.is_err());

    // Verify region1 still has its original name
    let regions = db.regions();
    assert!(regions.get_from_id("region1").is_some());
    assert!(regions.get_from_id("region2").is_some());

    Ok(())
}

#[test]
fn test_rename_after_remove_and_recreate() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create a region, write some data, then remove it
    let region1 = db.create_region_if_needed("temp")?;
    region1.write(b"Old data")?;
    region1.remove()?;

    // Create a new region with the same name
    let region2 = db.create_region_if_needed("temp")?;
    region2.write(b"New data")?;

    // Rename the new region
    region2.rename("renamed")?;

    // Verify the rename worked
    let regions = db.regions();
    assert!(regions.get_from_id("temp").is_none());
    assert!(regions.get_from_id("renamed").is_some());

    // Verify it has the new data (not old)
    let reader = region2.create_reader();
    assert_eq!(reader.read_all(), b"New data");

    Ok(())
}

#[test]
fn test_multiple_renames() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("name1")?;
    region.write(b"Data")?;

    // Rename multiple times
    region.rename("name2")?;
    region.rename("name3")?;
    region.rename("name4")?;

    // Verify final name
    let regions = db.regions();
    assert!(regions.get_from_id("name1").is_none());
    assert!(regions.get_from_id("name2").is_none());
    assert!(regions.get_from_id("name3").is_none());
    assert!(regions.get_from_id("name4").is_some());

    // Verify data is still intact
    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"Data");

    Ok(())
}

#[test]
fn test_rename_preserves_region_metadata() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("original")?;

    // Write large data to trigger expansion
    let large_data = vec![42u8; PAGE_SIZE * 2];
    region.write(&large_data)?;

    // Capture metadata before rename
    let (start_before, len_before, reserved_before, index_before) = {
        let meta = region.meta();
        (meta.start(), meta.len(), meta.reserved(), region.index())
    };

    // Rename
    region.rename("renamed")?;

    // Verify metadata preserved (except id)
    {
        let meta = region.meta();
        assert_eq!(meta.id(), "renamed");
        assert_eq!(meta.start(), start_before);
        assert_eq!(meta.len(), len_before);
        assert_eq!(meta.reserved(), reserved_before);
        assert_eq!(region.index(), index_before);
    }

    // Verify data is still intact
    let reader = region.create_reader();
    assert_eq!(reader.read_all(), &large_data[..]);

    Ok(())
}

#[test]
fn test_rename_with_special_characters() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("simple")?;

    // Rename with special characters (but not control characters)
    region.rename("name-with-dashes")?;
    assert!(db.regions().get_from_id("name-with-dashes").is_some());

    region.rename("name_with_underscores")?;
    assert!(db.regions().get_from_id("name_with_underscores").is_some());

    region.rename("name.with.dots")?;
    assert!(db.regions().get_from_id("name.with.dots").is_some());

    region.rename("name:with:colons")?;
    assert!(db.regions().get_from_id("name:with:colons").is_some());

    Ok(())
}

#[test]
fn test_concurrent_renames() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Arc::new(Database::open(temp.path())?);

    // Create regions upfront
    let regions: Vec<_> = (0..10)
        .map(|i| db.create_region_if_needed(&format!("region_{}", i)))
        .collect::<Result<Vec<_>>>()?;

    // Rename different regions concurrently
    let handles: Vec<_> = regions
        .into_iter()
        .enumerate()
        .map(|(i, region)| thread::spawn(move || region.rename(&format!("renamed_{}", i))))
        .collect();

    // Wait for all renames
    for handle in handles {
        handle.join().unwrap()?;
    }

    // Verify all renames succeeded
    let regions_lock = db.regions();
    for i in 0..10 {
        assert!(regions_lock.get_from_id(&format!("region_{}", i)).is_none());
        assert!(
            regions_lock
                .get_from_id(&format!("renamed_{}", i))
                .is_some()
        );
    }

    Ok(())
}

#[cfg(unix)]
#[test]
fn test_region_residency_hint() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("resident")?;

    assert!(region.prefers_mmap(0, 0));
    assert!(region.prefers_mmap(0, 1));

    let bytes = vec![42; 256 * 1024];
    region.write(&bytes)?;

    assert!(region.prefers_mmap(0, bytes.len()));
    assert!(region.prefers_mmap(bytes.len() - 1, 2));
    assert!(!region.prefers_mmap(bytes.len() - 1, 128 * 1024));

    Ok(())
}
