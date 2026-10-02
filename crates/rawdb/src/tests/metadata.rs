use std::fs;

use tempfile::TempDir;

use crate::{Database, PAGE_SIZE, Result};

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
