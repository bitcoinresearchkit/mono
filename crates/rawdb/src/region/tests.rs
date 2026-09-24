use std::fs;

use tempfile::TempDir;

use crate::{Database, PAGE_SIZE, Result};

#[test]
fn partial_relocation_defers_metadata_and_excludes_flush() -> Result<()> {
    let directory = TempDir::new()?;
    let db = Database::open(directory.path())?;
    let region = db.create_region_if_needed("moving")?;
    region.write(b"old bytes")?;
    db.create_region_if_needed("neighbor")?.write(b"kept")?;
    db.flush()?;
    let metadata = fs::read(directory.path().join("regions"))?;

    {
        let _access = region.0.access.write();
        let _writes = region.reserve_inner(&db, 2 * PAGE_SIZE, 1)?;
        assert!(db.inner.writes.try_write().is_none());
        assert_eq!(fs::read(directory.path().join("regions"))?, metadata);

        let mut meta = region.0.meta.write();
        // SAFETY: both the region and flush barrier remain locked above.
        unsafe { db.inner.data.write(meta.start() + 1, b"ther bytes") };
        unsafe { region.0.mark_dirty(1, 10) };
        meta.set_len(11);
        drop(meta);
        db.regions().update_bounds(region.index(), &region.meta());
    }
    db.flush()?;
    drop(region);
    drop(db);

    let db = Database::open(directory.path())?;
    assert_eq!(
        db.get_region("moving").unwrap().create_reader().read_all(),
        b"other bytes"
    );
    assert_eq!(
        db.get_region("neighbor")
            .unwrap()
            .create_reader()
            .read_all(),
        b"kept"
    );
    Ok(())
}
