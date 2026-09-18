#![cfg(all(feature = "derive", feature = "pco"))]

use std::any::TypeId;

use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, Bytes, Database, ImportableVec, Pco, PcoVec, ReadableVec, Result, Version,
    WritableVec,
};

// Test with a single generic parameter
#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct Wrapper<T>(T);

// Test with nested generics
#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct Container<T>(Wrapper<T>);

// Test with float type
#[derive(Debug, Clone, Copy, PartialEq, Pco)]
struct FloatWrapper<T>(T);

#[test]
fn test_derive_pco_with_single_generic() -> Result<()> {
    const { assert!(Wrapper::<u64>::IS_NATIVE_LAYOUT) };

    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let mut vec: PcoVec<usize, Wrapper<u64>> = PcoVec::import(&db, "test_u64", Version::TWO)?;

    vec.push(Wrapper(100));
    vec.push(Wrapper(200));
    vec.push(Wrapper(300));

    vec.write()?;

    let collected: Vec<Wrapper<u64>> = vec.collect();
    assert_eq!(collected.len(), 3);
    assert_eq!(collected[0], Wrapper(100));
    assert_eq!(collected[1], Wrapper(200));
    assert_eq!(collected[2], Wrapper(300));

    Ok(())
}

#[test]
fn test_derive_pco_with_different_types() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    // Test with u32
    {
        let mut vec: PcoVec<usize, Wrapper<u32>> = PcoVec::import(&db, "test_u32", Version::TWO)?;

        vec.push(Wrapper(42));
        vec.push(Wrapper(84));

        vec.write()?;

        let collected: Vec<Wrapper<u32>> = vec.collect();
        assert_eq!(collected, vec![Wrapper(42), Wrapper(84)]);
    }

    // Test with i64
    {
        let mut vec: PcoVec<usize, Wrapper<i64>> = PcoVec::import(&db, "test_i64", Version::TWO)?;

        vec.push(Wrapper(-100));
        vec.push(Wrapper(100));

        vec.write()?;

        let collected: Vec<Wrapper<i64>> = vec.collect();
        assert_eq!(collected, vec![Wrapper(-100), Wrapper(100)]);
    }

    Ok(())
}

// Test with nested generics
#[test]
fn test_derive_pco_with_nested_generics() -> Result<()> {
    const { assert!(Container::<u32>::IS_NATIVE_LAYOUT) };

    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let mut vec: PcoVec<usize, Container<u32>> = PcoVec::import(&db, "test_nested", Version::TWO)?;

    vec.push(Container(Wrapper(111)));
    vec.push(Container(Wrapper(222)));

    vec.write()?;

    let collected: Vec<Container<u32>> = vec.collect();
    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0], Container(Wrapper(111)));
    assert_eq!(collected[1], Container(Wrapper(222)));

    Ok(())
}

// Test with float type
#[test]
fn test_derive_pco_with_float_generic() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let mut vec: PcoVec<usize, FloatWrapper<f64>> =
        PcoVec::import(&db, "test_float", Version::TWO)?;

    vec.push(FloatWrapper(3.144));
    vec.push(FloatWrapper(2.71));

    vec.write()?;

    let collected: Vec<FloatWrapper<f64>> = vec.collect();
    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0], FloatWrapper(3.144));
    assert_eq!(collected[1], FloatWrapper(2.71));

    Ok(())
}

// Test that Pco NumberType is correctly propagated through generics
#[test]
fn test_pco_number_type_with_generic() {
    // Verify that the NumberType is correctly set
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
