use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Bytes, ImportOptions, ImportableVec, OverflowVec, OverflowVecValue,
    ReadableVec, Result, Stamp, Version, WritableVec,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TestValue(u64);

impl Bytes for TestValue {
    type Array = [u8; 8];

    fn to_bytes(&self) -> Self::Array {
        self.0.to_le_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self(u64::from_bytes(bytes)?))
    }
}

impl OverflowVecValue for TestValue {
    type Compact = u8;

    const VERSION: Version = Version::ONE;

    fn to_compact(&self) -> Option<Self::Compact> {
        u8::try_from(self.0).ok().filter(|value| *value < 128)
    }

    fn from_compact(compact: Self::Compact) -> Self {
        debug_assert!(compact < 128);
        Self(u64::from(compact))
    }

    fn overflow_index(compact: Self::Compact) -> Option<usize> {
        (compact >= 128).then_some(usize::from(compact & 127))
    }

    fn from_overflow_index(index: usize) -> Self::Compact {
        assert!(index < 128);
        128 | index as u8
    }
}

type V = OverflowVec<usize, TestValue>;

#[test]
fn large_range_decodes_inline_and_overflow_values() -> Result<()> {
    let temp = tempdir()?;
    let db = vecdb::Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "large", Version::ONE)?;
    let expected: Vec<_> = (0..70_000)
        .map(|index| {
            if index % 2_048 == 0 {
                TestValue(1_000 + index as u64)
            } else {
                TestValue((index % 128) as u64)
            }
        })
        .collect();

    for value in &expected {
        vec.push(*value);
    }
    vec.write()?;

    assert_eq!(vec.collect(), expected);
    assert_eq!(vec.read_only_clone().collect(), expected);
    Ok(())
}

#[test]
fn roundtrip_updates_holes_and_read_only_visibility() -> Result<()> {
    let temp = tempdir()?;
    let db = vecdb::Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "values", Version::ONE)?;
    let read_only = vec.read_only_clone();

    vec.push(TestValue(7));
    vec.push(TestValue(1_000));
    vec.push(TestValue(3));
    let reader = vec.reader();
    assert_eq!(vec.get_with_reader(1, &reader), Some(TestValue(1_000)));
    assert_eq!(read_only.len(), 0);

    vec.write()?;

    let mut cursor = vec.reader().cursor();
    cursor.advance(1);
    assert_eq!(cursor.next(), Some(TestValue(1_000)));
    assert_eq!(cursor.position(), 2);
    assert_eq!(cursor.remaining(), 1);
    assert_eq!(cursor.get(2), Some(TestValue(3)));

    assert_eq!(
        read_only.collect(),
        vec![TestValue(7), TestValue(1_000), TestValue(3)]
    );
    assert_eq!(vec.region_names().len(), 2);

    vec.update_many([(0, TestValue(5_000)), (1, TestValue(6_000))])?;
    vec.delete(1);
    assert_eq!(vec.holes().len(), 1);
    assert_eq!(vec.fill_first_hole_or_push(TestValue(9))?, 1);
    vec.delete(0);
    assert_eq!(vec.fill_first_hole_or_push(TestValue(7_000))?, 0);
    assert_eq!(
        vec.collect(),
        vec![TestValue(7_000), TestValue(9), TestValue(3)]
    );

    vec.write()?;
    drop(vec);

    let vec = V::import(&db, "values", Version::ONE)?;
    assert_eq!(
        vec.collect(),
        vec![TestValue(7_000), TestValue(9), TestValue(3)]
    );
    Ok(())
}

#[test]
fn rollback_and_truncation_keep_sidecar_in_sync() -> Result<()> {
    let temp = tempdir()?;
    let db = vecdb::Database::open(temp.path())?;
    let options = ImportOptions::new(&db, "rollback", Version::ONE).with_saved_stamped_changes(5);
    let mut vec = V::forced_import_with(options)?;

    vec.push(TestValue(1));
    vec.push(TestValue(1_000));
    vec.push(TestValue(2));
    vec.stamped_write_with_changes(Stamp::new(1))?;

    vec.truncate_if_needed_at(1)?;
    vec.push(TestValue(4_000));
    vec.stamped_write_with_changes(Stamp::new(2))?;
    assert_eq!(vec.collect(), vec![TestValue(1), TestValue(4_000)]);

    vec.rollback()?;
    assert_eq!(vec.stamp(), Stamp::new(1));
    assert_eq!(
        vec.collect(),
        vec![TestValue(1), TestValue(1_000), TestValue(2)]
    );
    Ok(())
}

#[test]
fn forced_version_reset_removes_data_and_holes() -> Result<()> {
    let temp = tempdir()?;
    let db = vecdb::Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "reset", Version::ONE)?;
    vec.push(TestValue(1_000));
    vec.push(TestValue(2_000));
    vec.write()?;
    vec.delete(0);
    vec.write()?;
    assert_eq!(vec.holes().len(), 1);
    drop(vec);

    let mut vec = V::forced_import(&db, "reset", Version::TWO)?;
    assert!(vec.is_empty());
    assert!(vec.holes().is_empty());
    assert_eq!(vec.fill_first_hole_or_push(TestValue(3_000))?, 0);
    vec.write()?;
    assert_eq!(vec.collect(), vec![TestValue(3_000)]);
    Ok(())
}
