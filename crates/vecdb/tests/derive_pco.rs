#![cfg(all(feature = "derive", feature = "pco"))]

use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, AnyVec, Bytes, Database, ImportableVec, Pco, PcoVec, ReadableVec, Result,
    Version, WritableVec,
};

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct Timestamp(u64);

#[test]
fn test_derive_pco_vec_value() -> Result<()> {
    const { assert!(Timestamp::IS_NATIVE_LAYOUT) };

    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let mut vec: PcoVec<usize, Timestamp> = PcoVec::import(&db, "test", Version::TWO)?;

    // Test push
    vec.push(Timestamp(12345));
    vec.push(Timestamp(67890));
    vec.push(Timestamp(111213));

    // Test write
    vec.write()?;

    // Test collect
    let collected: Vec<Timestamp> = vec.collect();
    assert_eq!(
        collected,
        vec![Timestamp(12345), Timestamp(67890), Timestamp(111213)]
    );

    // Test length
    assert_eq!(vec.len(), 3);

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct Price(f64);

#[test]
fn test_derive_with_float() -> Result<()> {
    const { assert!(Price::IS_NATIVE_LAYOUT) };

    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let mut vec: PcoVec<usize, Price> = PcoVec::import(&db, "prices", Version::TWO)?;

    vec.push(Price(19.99));
    vec.push(Price(29.99));
    vec.push(Price(39.99));

    vec.write()?;

    let collected: Vec<Price> = vec.collect();
    assert_eq!(collected.len(), 3);
    assert_eq!(collected[0], Price(19.99));
    assert_eq!(collected[1], Price(29.99));
    assert_eq!(collected[2], Price(39.99));

    Ok(())
}
