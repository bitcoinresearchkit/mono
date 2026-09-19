use std::fs;

use tempfile::TempDir;

use crate::{Database, PAGE_SIZE, Result};

use super::setup_test_db;

#[test]
fn test_database_creation() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Database should start empty
    assert_eq!(db.regions().len(), 0);
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
    assert_eq!(regions.len(), 1);
    assert!(regions.get("test_region").is_some());

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
    assert_eq!(db.regions().len(), 1);

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
    assert_eq!(db.regions().len(), 3);
    assert_eq!(db.layout().start_to_region().len(), 3);

    // Verify data integrity
    let mmap = fs::read(db.path().join("data"))?;

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
fn test_remove_region() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    let index = region.index();

    region.write(b"Hello")?;

    // Remove region
    region.remove()?;

    // Verify removal
    let regions = db.regions();
    assert!(regions.get("test").is_none());
    assert!(regions.iter().all(|region| region.index() != index));

    db.flush()?; // Make hole available

    // Layout should have a hole now
    let layout = db.layout();
    assert_eq!(layout.start_to_region().len(), 0);
    assert_eq!(layout.start_to_hole().len(), 1);

    Ok(())
}

#[test]
fn test_remove_regions_preserves_others() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let _ = db.create_region_if_needed("keep1")?;
    let _ = db.create_region_if_needed("remove1")?;
    let _ = db.create_region_if_needed("keep2")?;
    let _ = db.create_region_if_needed("remove2")?;

    db.remove_region("remove1")?;
    db.remove_region("remove2")?;

    let regions = db.regions();
    assert_eq!(regions.len(), 2);
    assert!(regions.get("keep1").is_some());
    assert!(regions.get("keep2").is_some());
    assert!(regions.get("remove1").is_none());
    assert!(regions.get("remove2").is_none());

    Ok(())
}

#[test]
fn test_retain_accessed_regions() -> Result<()> {
    let temp = TempDir::new()?;
    {
        let db = Database::open(temp.path())?;
        let _ = db.create_region_if_needed("kept_by_get")?;
        let _ = db.create_region_if_needed("kept_by_create")?;
        let _ = db.create_region_if_needed("stale")?;
    }

    let db = Database::open(temp.path())?;
    let _ = db.get_region("kept_by_get").unwrap();
    let _ = db.create_region_if_needed("kept_by_create")?;
    let _ = db.create_region_if_needed("new")?;
    db.retain_accessed_regions()?;

    let regions = db.regions();
    assert_eq!(regions.len(), 3);
    assert!(regions.get("kept_by_get").is_some());
    assert!(regions.get("kept_by_create").is_some());
    assert!(regions.get("new").is_some());
    assert!(regions.get("stale").is_none());

    Ok(())
}

#[test]
fn test_retain_accessed_regions_shrinks_metadata_and_preserves_hole_reuse() -> Result<()> {
    let temp = TempDir::new()?;
    let regions = fs::File::create(temp.path().join("regions"))?;
    regions.set_len(50_000 * PAGE_SIZE as u64)?;
    let db = Database::open(temp.path())?;

    let _ = db.create_region_if_needed("keep0")?;
    let _ = db.create_region_if_needed("remove1")?;
    let _ = db.create_region_if_needed("keep2")?;

    drop(db);
    let db = Database::open(temp.path())?;
    let _ = db.get_region("keep0").unwrap();
    let _ = db.get_region("keep2").unwrap();
    db.retain_accessed_regions()?;
    assert_eq!(
        fs::metadata(temp.path().join("regions"))?.len(),
        3 * PAGE_SIZE as u64
    );

    drop(db);
    let db = Database::open(temp.path())?;
    assert_eq!(db.regions().len(), 2);
    assert_eq!(db.create_region_if_needed("reused")?.index(), 1);

    Ok(())
}

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
    let retrieved = regions.get(&long_name);
    assert!(retrieved.is_some());

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
