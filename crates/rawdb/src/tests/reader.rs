use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::Result;

use super::setup_test_db;

#[test]
fn test_reader() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Hello, World!")?;

    let reader = region.create_reader();
    assert_eq!(reader.read_all(), b"Hello, World!");
    assert_eq!(reader.read(0, 5), b"Hello");
    assert_eq!(reader.read(7, 5), b"World");

    Ok(())
}

#[test]
fn test_reader_read_from() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"0123456789ABCDEF")?;

    let reader = region.create_reader();

    assert_eq!(reader.read_from(5), b"56789ABCDEF");
    assert_eq!(reader.read_from(0), b"0123456789ABCDEF");
    assert!(reader.read_from(reader.len()).is_empty());
    for offset in [reader.len() + 1, usize::MAX] {
        assert!(catch_unwind(AssertUnwindSafe(|| reader.read_from(offset))).is_err());
    }

    Ok(())
}

#[test]
#[should_panic]
fn test_reader_bounds_check() {
    let (db, _temp) = setup_test_db().unwrap();

    let region = db.create_region_if_needed("test").unwrap();
    region.write(b"Short").unwrap();

    let reader = region.create_reader();

    // This should panic due to bounds check
    let _ = reader.read(0, 100);
}

#[test]
fn test_reader_outlives_region_variable() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let reader = {
        let region = db.create_region_if_needed("test")?;
        region.write(b"Hello from dropped region")?;
        region.create_reader()
        // region is dropped here, but reader should still be valid
    };

    // Reader should still work because it holds its own Arc references
    assert_eq!(reader.read_all(), b"Hello from dropped region");
    assert_eq!(reader.read(0, 5), b"Hello");

    Ok(())
}

#[test]
fn test_reader_outlives_database_variable() -> Result<()> {
    let (_temp, reader) = {
        let (db, temp) = setup_test_db()?;
        let region = db.create_region_if_needed("test")?;
        region.write(b"Persisted data")?;
        let reader = region.create_reader();
        // Both db and region go out of scope, but reader holds Arc clones
        (temp, reader)
    };

    // Reader should still work
    assert_eq!(reader.read_all(), b"Persisted data");

    Ok(())
}

#[test]
fn test_multiple_readers_from_same_region() -> Result<()> {
    let (db, _temp) = setup_test_db()?;

    let region = db.create_region_if_needed("test")?;
    region.write(b"Shared data")?;

    // Create multiple readers
    let reader1 = region.create_reader();
    let reader2 = region.create_reader();
    let reader3 = region.create_reader();

    // All should read the same data
    assert_eq!(reader1.read_all(), b"Shared data");
    assert_eq!(reader2.read_all(), b"Shared data");
    assert_eq!(reader3.read_all(), b"Shared data");

    // Drop in different order
    drop(reader2);
    assert_eq!(reader1.read_all(), b"Shared data");
    assert_eq!(reader3.read_all(), b"Shared data");

    Ok(())
}
