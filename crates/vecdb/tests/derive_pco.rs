#![cfg(all(feature = "derive", feature = "pco"))]

use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, AnyVec, Bytes, Database, ImportableVec, Pco, PcoVec, PcoVecValue, ReadableVec,
    Result, Version, WritableVec,
};

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct Timestamp(u64);

fn roundtrip<T>(name: &str, values: &[T]) -> Result<()>
where
    T: PartialEq + PcoVecValue,
{
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    let mut vec: PcoVec<usize, T> = PcoVec::import(&db, name, Version::TWO)?;
    for value in values {
        vec.push(value.clone());
    }
    vec.write()?;
    assert_eq!(vec.collect().as_slice(), values);
    assert_eq!(vec.len(), values.len());
    Ok(())
}

#[test]
fn test_derive_pco_vec_value() -> Result<()> {
    const { assert!(Timestamp::IS_NATIVE_LAYOUT) };

    roundtrip(
        "test",
        &[Timestamp(12345), Timestamp(67890), Timestamp(111213)],
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct Price(f64);

#[test]
fn test_derive_with_float() -> Result<()> {
    const { assert!(Price::IS_NATIVE_LAYOUT) };

    roundtrip("prices", &[Price(19.99), Price(29.99), Price(39.99)])
}
