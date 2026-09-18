#![cfg(feature = "derive")]

use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, Bytes, BytesVec, Database, ImportableVec, ReadableVec, Result, Version,
    WritableVec,
};

// Test with a single generic parameter
#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct Wrapper<T>(T);

// Test with nested generics
#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct Container<T>(Wrapper<T>);

// Test with float type
#[derive(Debug, Clone, Copy, PartialEq, Bytes)]
struct FloatWrapper<T>(T);

#[test]
fn test_derive_bytes_with_single_generic() -> Result<()> {
    const { assert!(Wrapper::<u64>::IS_NATIVE_LAYOUT) };

    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let mut vec: BytesVec<usize, Wrapper<u64>> = BytesVec::import(&db, "test_u64", Version::TWO)?;

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
fn test_derive_bytes_with_different_types() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    // Test with u32
    {
        let mut vec: BytesVec<usize, Wrapper<u32>> =
            BytesVec::import(&db, "test_u32", Version::TWO)?;

        vec.push(Wrapper(42));
        vec.push(Wrapper(84));

        vec.write()?;

        let collected: Vec<Wrapper<u32>> = vec.collect();
        assert_eq!(collected, vec![Wrapper(42), Wrapper(84)]);
    }

    // Test with i64
    {
        let mut vec: BytesVec<usize, Wrapper<i64>> =
            BytesVec::import(&db, "test_i64", Version::TWO)?;

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
fn test_derive_bytes_with_nested_generics() -> Result<()> {
    const { assert!(Container::<u32>::IS_NATIVE_LAYOUT) };

    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let mut vec: BytesVec<usize, Container<u32>> =
        BytesVec::import(&db, "test_nested", Version::TWO)?;

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
fn test_derive_bytes_with_float_generic() -> Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let mut vec: BytesVec<usize, FloatWrapper<f64>> =
        BytesVec::import(&db, "test_float", Version::TWO)?;

    vec.push(FloatWrapper(3.144));
    vec.push(FloatWrapper(2.71));

    vec.write()?;

    let collected: Vec<FloatWrapper<f64>> = vec.collect();
    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0], FloatWrapper(3.144));
    assert_eq!(collected[1], FloatWrapper(2.71));

    Ok(())
}
