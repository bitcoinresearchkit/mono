#![cfg(feature = "derive")]

use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, Bytes, BytesVec, Database, ImportableVec, ReadableVec, Result, VecValue, Version,
    WritableVec,
};

fn roundtrip<T>(db: &Database, name: &str, values: &[T]) -> Result<()>
where
    T: Bytes + PartialEq + VecValue,
{
    let mut vec: BytesVec<usize, T> = BytesVec::import(db, name, Version::TWO)?;
    for value in values {
        vec.push(value.clone());
    }
    vec.write()?;
    assert_eq!(vec.collect().as_slice(), values);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct Wrapper<T>(T);

#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct Container<T>(Wrapper<T>);

#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct FloatWrapper<T>(T);

#[test]
fn test_derive_bytes_with_single_generic() -> Result<()> {
    const { assert!(Wrapper::<u64>::IS_NATIVE_LAYOUT) };

    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    roundtrip(
        &db,
        "test_u64",
        &[Wrapper(100_u64), Wrapper(200_u64), Wrapper(300_u64)],
    )
}

#[test]
fn test_derive_bytes_with_different_types() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    roundtrip(&db, "test_u32", &[Wrapper(42_u32), Wrapper(84_u32)])?;
    roundtrip(&db, "test_i64", &[Wrapper(-100_i64), Wrapper(100_i64)])
}

#[test]
fn test_derive_bytes_with_nested_generics() -> Result<()> {
    const { assert!(Container::<u32>::IS_NATIVE_LAYOUT) };

    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    roundtrip(
        &db,
        "test_nested",
        &[Container(Wrapper(111_u32)), Container(Wrapper(222_u32))],
    )
}

#[test]
fn test_derive_bytes_with_float_generic() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    roundtrip(
        &db,
        "test_float",
        &[FloatWrapper(3.144_f64), FloatWrapper(2.71_f64)],
    )
}
