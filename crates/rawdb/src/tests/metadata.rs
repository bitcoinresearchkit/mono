use std::fs;

use tempfile::TempDir;

use crate::{Database, Error, PAGE_SIZE, Result};

use super::setup_test_db;

#[test]
fn updating_bounds_preserves_identity_and_reopens_after_relocation() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let id = "x".repeat(1024);
    let region = db.create_region_if_needed(&id)?;
    region.write(b"abcdefgh")?;
    let neighbor = db.create_region_if_needed("neighbor")?;
    neighbor.write(b"neighbor")?;
    db.flush()?;
    let before = fs::read(dir.path().join("regions"))?;

    region.reserve_capacity(PAGE_SIZE * 2)?;
    region.truncate_write(2, b"xyz")?;
    assert!(region.meta().start() > neighbor.meta().start());
    db.flush()?;
    let after = fs::read(dir.path().join("regions"))?;
    assert_eq!(&before[24..PAGE_SIZE], &after[24..PAGE_SIZE]);
    drop(region);
    drop(neighbor);
    drop(db);

    let db = Database::open(dir.path())?;
    let region = db.get_region(&id).unwrap();
    assert_eq!(region.meta().reserved(), PAGE_SIZE * 2);
    assert_eq!(region.create_reader().read_all(), b"abxyz");
    assert_eq!(
        db.get_region("neighbor")
            .unwrap()
            .create_reader()
            .read_all(),
        b"neighbor"
    );
    Ok(())
}

#[test]
fn test_open_rejects_invalid_nonempty_metadata() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let region = db.create_region_if_needed("test")?;
    db.flush()?;
    drop(region);
    drop(db);

    let path = temp.path().join("regions");
    let mut bytes = fs::read(&path)?;
    bytes[16..24].copy_from_slice(&0u64.to_le_bytes());
    fs::write(path, bytes)?;

    assert!(matches!(
        Database::open(temp.path()),
        Err(Error::CorruptedMetadata(_))
    ));
    Ok(())
}

#[test]
fn test_open_rejects_nonzero_vacant_metadata() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let first = db.create_region_if_needed("first")?;
    let second = db.create_region_if_needed("second")?;
    drop(first);
    db.remove_region("first")?;
    db.flush()?;
    drop(second);
    drop(db);

    let path = temp.path().join("regions");
    let mut bytes = fs::read(&path)?;
    bytes[PAGE_SIZE - 1] = 1;
    fs::write(path, bytes)?;

    assert!(matches!(
        Database::open(temp.path()),
        Err(Error::CorruptedMetadata(_))
    ));
    Ok(())
}

#[test]
fn test_open_rejects_overlapping_regions() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let first = db.create_region_if_needed("first")?;
    let second = db.create_region_if_needed("second")?;
    db.flush()?;
    drop(first);
    drop(second);
    drop(db);

    let path = temp.path().join("regions");
    let mut bytes = fs::read(&path)?;
    let second_start = PAGE_SIZE;
    bytes[second_start..second_start + 8].copy_from_slice(&0u64.to_le_bytes());
    fs::write(path, bytes)?;

    assert!(matches!(
        Database::open(temp.path()),
        Err(Error::CorruptedMetadata(_))
    ));
    Ok(())
}

#[test]
fn test_open_rejects_duplicate_region_ids() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let first = db.create_region_if_needed("first")?;
    let second = db.create_region_if_needed("second")?;
    db.flush()?;
    drop(first);
    drop(second);
    drop(db);

    let path = temp.path().join("regions");
    let mut bytes = fs::read(&path)?;
    let second_start = PAGE_SIZE;
    bytes[second_start + 24..second_start + 32].copy_from_slice(&5u64.to_le_bytes());
    bytes[second_start + 32..second_start + 37].copy_from_slice(b"first");
    fs::write(path, bytes)?;

    assert!(matches!(
        Database::open(temp.path()),
        Err(Error::CorruptedMetadata(_))
    ));
    Ok(())
}

#[test]
fn test_open_rejects_region_beyond_data_file() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let region = db.create_region_if_needed("test")?;
    db.flush()?;
    drop(region);
    drop(db);

    let data_len = fs::metadata(temp.path().join("data"))?.len();
    let path = temp.path().join("regions");
    let mut bytes = fs::read(&path)?;
    bytes[..8].copy_from_slice(&data_len.to_le_bytes());
    fs::write(path, bytes)?;

    assert!(matches!(
        Database::open(temp.path()),
        Err(Error::CorruptedMetadata(_))
    ));
    Ok(())
}
