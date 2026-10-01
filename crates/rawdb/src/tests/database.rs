use std::fs;

use tempfile::TempDir;

use crate::{Database, PAGE_SIZE, Result};

use super::setup_test_db;

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
