use std::fs;

use tempfile::TempDir;

use crate::{Database, Result};

use super::setup_test_db;

#[test]
fn first_flush_after_reopen_synchronizes_metadata_without_new_writes() -> Result<()> {
    let temp = TempDir::new()?;
    {
        let db = Database::open(temp.path())?;
        db.create_region_if_needed("values")?.write(b"unsynced")?;
    }
    let db = Database::open(temp.path())?;
    // Region dirty ranges belong to the previous process. Reopening must still
    // force the data-file sync and then the metadata-file sync on first flush.
    assert_eq!(db.flush_inner()?, (0, true));
    assert_eq!(db.flush_inner()?, (0, false));
    assert_eq!(
        db.get_region("values").unwrap().create_reader().read_all(),
        b"unsynced"
    );
    Ok(())
}

#[test]
fn test_persistence() -> Result<()> {
    let temp = TempDir::new()?;
    let path = temp.path();

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
        let region = regions.get("persistent").expect("Region should persist");

        let meta = region.meta();
        assert_eq!(meta.len(), 14);
        let start = meta.start();
        drop(meta);

        let mmap = fs::read(db.path().join("data"))?;
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
    let region = regions.get("empty").expect("empty region should persist");
    assert_eq!(region.meta().len(), 0);

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
        assert!(regions.get("keep1").is_some());
        assert!(regions.get("remove").is_none());
        assert!(regions.get("keep2").is_some());

        let r1 = regions.get("keep1").unwrap();
        let r3 = regions.get("keep2").unwrap();

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
        let r = regions.get("test").unwrap();

        let reader = r.create_reader();
        assert_eq!(reader.read_all(), b"Version 1 Version 2 Version 3");
    }

    Ok(())
}

#[test]
fn test_remove_all_regions() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    // Create some regions
    let _ = db.create_region_if_needed("keep1")?;
    let _ = db.create_region_if_needed("keep2")?;
    let _ = db.create_region_if_needed("remove1")?;

    for name in ["keep1", "keep2", "remove1"] {
        db.remove_region(name)?;
    }

    let regions = db.regions();
    assert_eq!(regions.len(), 0);

    Ok(())
}
