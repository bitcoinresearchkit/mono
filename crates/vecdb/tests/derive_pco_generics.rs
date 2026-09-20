#![cfg(all(feature = "derive", feature = "pco"))]

use std::any::TypeId;

use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, Bytes, Database, ImportableVec, Pco, PcoVec, PcoVecValue, ReadableVec, Result,
    Version, WritableVec,
};

fn roundtrip<T>(db: &Database, name: &str, values: &[T]) -> Result<()>
where
    T: PartialEq + PcoVecValue,
{
    let mut vec: PcoVec<usize, T> = PcoVec::import(db, name, Version::TWO)?;
    for value in values {
        vec.push(value.clone());
    }
    vec.write()?;
    assert_eq!(vec.collect().as_slice(), values);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct Wrapper<T>(T);

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct Container<T>(Wrapper<T>);

#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct FloatWrapper<T>(T);

#[test]
fn test_derive_pco_with_single_generic() -> Result<()> {
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
fn test_derive_pco_with_different_types() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    roundtrip(&db, "test_u32", &[Wrapper(42_u32), Wrapper(84_u32)])?;
    roundtrip(&db, "test_i64", &[Wrapper(-100_i64), Wrapper(100_i64)])
}

#[test]
fn test_derive_pco_with_nested_generics() -> Result<()> {
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
fn test_derive_pco_with_float_generic() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    roundtrip(
        &db,
        "test_float",
        &[FloatWrapper(3.144_f64), FloatWrapper(2.71_f64)],
    )
}

#[test]
fn test_pco_number_type_with_generic() {
    assert_eq!(
        TypeId::of::<<Wrapper<u64> as Pco>::NumberType>(),
        TypeId::of::<u64>()
    );

    assert_eq!(
        TypeId::of::<<Wrapper<f64> as Pco>::NumberType>(),
        TypeId::of::<f64>()
    );

    assert_eq!(
        TypeId::of::<<Container<u32> as Pco>::NumberType>(),
        TypeId::of::<u32>()
    );
}
