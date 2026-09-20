use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
};

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Bytes, BytesVec, Database, Error, ImportOptions, ImportableVec,
    MutableVec, ReadableVec, Result, Stamp, Version, WritableVec,
};

#[cfg(feature = "lz4")]
use vecdb::LZ4Vec;
#[cfg(feature = "pco")]
use vecdb::PcoVec;

#[derive(Debug, Clone, PartialEq)]
struct PanicValue(u64);

impl Bytes for PanicValue {
    type Array = [u8; 8];

    fn to_bytes(&self) -> Self::Array {
        assert_ne!(self.0, u64::MAX, "injected encoding failure");
        self.0.to_le_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self(u64::from_bytes(bytes)?))
    }
}

fn encoding_failure<V>() -> Result<()>
where
    V: ImportableVec + WritableVec<usize, PanicValue>,
{
    let dir = tempdir()?;
    let db = Database::open(dir.path())?;
    let mut values = V::import(&db, "values", Version::ONE)?;
    values.push(PanicValue(42));
    values.stamped_write(Stamp::new(1))?;
    values.push(PanicValue(u64::MAX));
    assert!(catch_unwind(AssertUnwindSafe(|| values.stamped_write(Stamp::new(2)))).is_err());
    assert_eq!(values.stamp(), Stamp::new(1));
    assert!(matches!(values.write(), Err(Error::WriteFailed)));
    assert!(matches!(values.flush(), Err(Error::WriteFailed)));
    assert!(matches!(values.reset(), Err(Error::WriteFailed)));
    assert!(matches!(
        values.truncate_if_needed_at(0),
        Err(Error::WriteFailed)
    ));
    assert!(matches!(values.rollback(), Err(Error::WriteFailed)));
    assert!(catch_unwind(AssertUnwindSafe(|| values.reset_unsaved())).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| values.push(PanicValue(3)))).is_err());
    Ok(())
}

#[test]
fn raw_encoding_panic_fences_the_writer() -> Result<()> {
    encoding_failure::<BytesVec<usize, PanicValue>>()
}

#[cfg(feature = "lz4")]
#[test]
fn compressed_encoding_panic_fences_the_writer() -> Result<()> {
    encoding_failure::<LZ4Vec<usize, PanicValue>>()
}

#[test]
fn late_holes_failure_fences_the_whole_mutable_write() -> Result<()> {
    let dir = tempdir()?;
    let db = Database::open(dir.path())?;
    let mut values = MutableVec::<BytesVec<usize, u64>>::import(&db, "values", Version::ONE)?;
    values.push(10);
    values.push(20);
    values.delete_at(0);
    values.stamped_write(Stamp::new(1))?;
    let holes = db.get_region("values/usize_holes").expect("stored holes");
    values.update_at(0, 99)?;
    // The value write succeeds before removing the still-referenced sidecar fails.
    assert!(values.stamped_write(Stamp::new(2)).is_err());
    assert_eq!(values.stamp(), Stamp::new(1));
    drop(holes);
    assert!(matches!(values.write(), Err(Error::WriteFailed)));
    assert!(matches!(
        values.stamped_write_with_changes(Stamp::new(2)),
        Err(Error::WriteFailed)
    ));
    assert!(matches!(values.update_at(1, 30), Err(Error::WriteFailed)));
    assert!(matches!(values.reset(), Err(Error::WriteFailed)));
    Ok(())
}

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

#[cfg(feature = "pco")]
#[test]
fn compressed_reset_rewrites_data_and_pages_on_reopen() -> Result<()> {
    let dir = tempdir()?;
    {
        let db = Database::open(dir.path())?;
        let mut values = PcoVec::<usize, u64>::import(&db, "values", Version::ONE)?;
        for value in 0..2049 {
            values.push(value);
        }
        values.flush()?;
        values.reset()?;
        values.flush()?;
    }
    let db = Database::open(dir.path())?;
    let values = PcoVec::<usize, u64>::import(&db, "values", Version::ONE)?;
    assert!(values.is_empty());
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
