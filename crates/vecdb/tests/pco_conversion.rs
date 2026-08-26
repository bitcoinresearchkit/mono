#![cfg(feature = "pco")]

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Bytes, Database, Error, ImportableVec, Pco, PcoVec, ReadableVec, Version,
    WritableVec,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum CheckedValue {
    One = 1,
    Two = 2,
}

impl Bytes for CheckedValue {
    type Array = [u8; 1];

    fn to_bytes(&self) -> Self::Array {
        [*self as u8]
    }

    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        match bytes {
            [1] => Ok(Self::One),
            [2] => Ok(Self::Two),
            _ => Err(Error::InvalidArgument("invalid CheckedValue")),
        }
    }
}

// SAFETY: The non-transparent conversion validates every decoded discriminant.
unsafe impl Pco for CheckedValue {
    type NumberType = u8;

    fn to_number(self) -> Self::NumberType {
        self as u8
    }

    fn from_number(value: Self::NumberType) -> vecdb::Result<Self> {
        Self::from_bytes(&[value])
    }
}

#[test]
fn non_transparent_values_roundtrip_through_compressed_pages() -> vecdb::Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = PcoVec::<usize, CheckedValue>::import(&db, "checked", Version::ONE)?;
    let expected = (0..20_000)
        .map(|index| {
            if index % 2 == 0 {
                CheckedValue::One
            } else {
                CheckedValue::Two
            }
        })
        .collect::<Vec<_>>();

    for &value in &expected {
        vec.push(value);
    }
    vec.write()?;

    assert_eq!(vec.collect(), expected);
    assert!(CheckedValue::from_number(0).is_err());
    Ok(())
}
