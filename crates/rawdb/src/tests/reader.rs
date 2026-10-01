use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::Result;

use super::setup_test_db;

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
