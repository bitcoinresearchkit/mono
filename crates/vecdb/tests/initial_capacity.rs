use std::ops::Add;

use rawdb::{Database, PAGE_SIZE};
use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, BytesVec, HEADER_OFFSET, ImportableVec, PrintableIndex, VecIndex, Version,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CapacityIndex(usize);

impl PrintableIndex for CapacityIndex {
    fn to_string() -> &'static str {
        "capacity_index"
    }

    fn to_possible_strings() -> &'static [&'static str] {
        &["capacity_index"]
    }
}

impl VecIndex for CapacityIndex {
    const INITIAL_CAPACITY: usize = 10_000;
}

impl From<usize> for CapacityIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<CapacityIndex> for usize {
    fn from(value: CapacityIndex) -> Self {
        value.0
    }
}

impl Add<usize> for CapacityIndex {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[test]
fn index_initial_capacity_defaults_to_zero() {
    assert_eq!(<usize as VecIndex>::INITIAL_CAPACITY, 0);
}

#[test]
fn index_initial_capacity_is_reserved_and_reused() -> vecdb::Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let vec = BytesVec::<CapacityIndex, u32>::forced_import(&db, "values", Version::ONE)?;
    let expected = (HEADER_OFFSET + 10_000 * size_of::<u32>()).next_multiple_of(PAGE_SIZE);
    assert_eq!(vec.region().meta().len(), HEADER_OFFSET);
    assert_eq!(vec.region().meta().reserved(), expected);
    drop(vec);

    let vec = BytesVec::<CapacityIndex, u32>::forced_import(&db, "values", Version::ONE)?;
    assert_eq!(vec.region().meta().reserved(), expected);

    Ok(())
}

#[cfg(feature = "pco")]
#[test]
fn compressed_vec_uses_index_initial_capacity() -> vecdb::Result<()> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;

    let vec =
        vecdb::PcoVec::<CapacityIndex, u32>::forced_import(&db, "compressed_values", Version::ONE)?;
    let expected = (HEADER_OFFSET + 10_000 * size_of::<u32>()).next_multiple_of(PAGE_SIZE);
    assert_eq!(vec.region().meta().len(), HEADER_OFFSET);
    assert_eq!(vec.region().meta().reserved(), expected);
    Ok(())
}
