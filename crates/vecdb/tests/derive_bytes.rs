#![cfg(feature = "derive")]

use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, AnyVec, Bytes, BytesVec, Database, HEADER_OFFSET, ImportableVec, ReadableVec,
    Result, Version, WritableVec,
};

#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct Timestamp(u64);

#[test]
fn test_derive_bytes_vec_value() -> Result<()> {
    const { assert!(Timestamp::IS_NATIVE_LAYOUT) };

    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    let values = [Timestamp(12345), Timestamp(67890), Timestamp(111213)];
    let mut vec: BytesVec<usize, Timestamp> = BytesVec::import(&db, "test", Version::TWO)?;
    for &value in &values {
        vec.push(value);
    }
    vec.write()?;

    let expected = [
        12_345_u64.to_le_bytes(),
        67_890_u64.to_le_bytes(),
        111_213_u64.to_le_bytes(),
    ]
    .concat();
    assert_eq!(
        &vec.region().create_reader().read_all()[HEADER_OFFSET..],
        expected
    );
    assert_eq!(vec.collect().as_slice(), values);
    assert_eq!(vec.len(), values.len());
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct Price(f64);

#[test]
fn test_derive_with_float() -> Result<()> {
    const { assert!(Price::IS_NATIVE_LAYOUT) };

    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    let values = [Price(19.99), Price(29.99), Price(39.99)];
    let mut vec: BytesVec<usize, Price> = BytesVec::import(&db, "prices", Version::TWO)?;
    for &value in &values {
        vec.push(value);
    }
    vec.write()?;
    assert_eq!(vec.collect().as_slice(), values);
    Ok(())
}
