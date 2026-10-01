use tempfile::TempDir;

use crate::{Database, Result};

#[test]
fn reopening_starts_clean_and_new_writes_are_tracked() -> Result<()> {
    let temp = TempDir::new()?;
    {
        let db = Database::open(temp.path())?;
        db.create_region_if_needed("values")?.write(b"unsynced")?;
    }
    let db = Database::open(temp.path())?;
    // Dirty tracking starts with this owner, as it does on main.
    assert_eq!(db.flush_inner(&db.inner.writes.write())?, (0, false));
    let region = db.get_region("values").unwrap();
    assert_eq!(region.create_reader().read_all(), b"unsynced");
    region.write(b" append")?;
    assert_eq!(db.flush_inner(&db.inner.writes.write())?, (1, true));
    assert_eq!(db.flush_inner(&db.inner.writes.write())?, (0, false));
    Ok(())
}

#[test]
fn test_empty_region_persistence() -> Result<()> {
    let temp = TempDir::new()?;

    {
        let db = Database::open(temp.path())?;
        let _ = db.create_region_if_needed("empty")?;
        assert_eq!(db.flush_inner(&db.inner.writes.write())?, (0, true));
        assert_eq!(db.flush_inner(&db.inner.writes.write())?, (0, false));
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
