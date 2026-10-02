use std::fs;

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, Error, ImportOptions, ImportableVec, MutableVec, ReadableVec,
    Result, Stamp, Version, WritableVec,
};

#[test]
fn failed_undo_publication_preserves_the_previous_file_and_values() -> Result<()> {
    let dir = tempdir()?;
    let db = Database::open(dir.path())?;
    let options = ImportOptions::new(&db, "values", Version::ONE).with_saved_stamped_changes(1);
    let mut values = MutableVec::<BytesVec<usize, u64>>::import_with(options)?;
    values.push(10);
    values.stamped_write_with_changes(Stamp::new(1))?;
    let published = values.read_only_clone();
    let changes = dir.path().join("changes/values/usize");
    let previous = fs::read(changes.join("1"))?;
    fs::create_dir(changes.join("2"))?;
    values.update_at(0, 20)?;
    assert!(values.stamped_write_with_changes(Stamp::new(2)).is_err());
    assert_eq!(published.collect(), [10]);
    assert_eq!(values.stamp(), Stamp::new(1));
    assert_eq!(fs::read(changes.join("1"))?, previous);
    assert!(matches!(values.write(), Err(Error::WriteFailed)));
    Ok(())
}

#[test]
fn flush_synchronizes_previous_writes_and_header_only_updates() -> Result<()> {
    let dir = tempdir()?;
    let db = Database::open(dir.path())?;
    let mut values = BytesVec::<usize, u64>::import(&db, "values", Version::ONE)?;
    values.push(42);
    values.write()?;
    values.flush()?;
    assert_eq!(db.flush()?, 0, "flush must include previously written data");
    values.update_stamp(Stamp::new(7));
    values.flush()?;
    assert_eq!(db.flush()?, 0, "flush must include header-only changes");
    drop((values, db));
    let db = Database::open(dir.path())?;
    let values = BytesVec::<usize, u64>::import(&db, "values", Version::ONE)?;
    assert_eq!(values.collect(), [42]);
    assert_eq!(values.stamp(), Stamp::new(7));
    Ok(())
}
