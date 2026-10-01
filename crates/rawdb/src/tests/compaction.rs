use std::fs;

use tempfile::TempDir;

use crate::{Database, PAGE_SIZE, Result};

use super::{allocated_bytes, setup_test_db};

#[test]
fn repeated_compaction_preserves_the_partial_page() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("partial_page")?;
    region.write(&vec![1; 2 * PAGE_SIZE])?;
    db.flush()?;

    for _ in 0..2 {
        region.truncate(10)?;
        db.compact()?;
        assert_eq!(region.meta().reserved(), 2 * PAGE_SIZE);
        assert_eq!(
            fs::read(db.path().join("data"))?[PAGE_SIZE - 1..=PAGE_SIZE],
            [1, 0]
        );
        db.flush()?;
    }
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
    let allocated_before = allocated_bytes(&db)?;

    region.truncate(100)?;
    db.compact()?;
    let allocated_after = allocated_bytes(&db)?;

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
        allocated_bytes(&db)?
    };

    let db = Database::open(temp.path())?;
    db.compact()?;
    let allocated_after = allocated_bytes(&db)?;

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
    let allocated_before = allocated_bytes(&db)?;

    removed.remove()?;
    db.compact()?;
    let allocated_after = allocated_bytes(&db)?;

    assert!(allocated_after < allocated_before);
    assert_eq!(retained.create_reader().read_all(), b"retained");

    Ok(())
}
