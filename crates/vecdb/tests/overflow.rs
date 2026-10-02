use std::fs;

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Bytes, Database, Error, ImportOptions, ImportableVec, OverflowVec,
    OverflowVecValue, ReadableVec, Result, Stamp, Version, WritableVec,
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

fn assert_sorted_reads(
    source: &impl ReadableVec<usize, TestValue>,
    expected: &[Option<TestValue>],
) {
    for indices in [
        vec![],
        vec![usize::MAX],
        vec![
            0,
            0,
            1,
            2,
            210,
            211,
            211,
            4095,
            4096,
            10_010,
            10_011,
            10_012,
            usize::MAX,
        ],
        (0..expected.len() + 1).collect(),
        (0..expected.len())
            .step_by(13)
            .flat_map(|i| [i, i])
            .collect(),
    ] {
        let values: Vec<_> = indices
            .iter()
            .filter_map(|&i| expected.get(i).copied().flatten())
            .collect();
        let mut actual = vec![TestValue(99)];
        source.read_sorted_into_at(&indices, &mut actual);
        assert_eq!(&actual[1..], values);
    }
}

#[test]
fn sorted_reads_preserve_holes_staged_sidecar_reuse_truncation_and_reopen() -> Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut source = OverflowVec::<usize, TestValue>::import(&db, "sorted", Version::ONE)?;
    let mut expected: Vec<_> = (0..10_011)
        .map(|i| {
            Some(TestValue(if i % 211 == 0 {
                1_000_000 + i as u64
            } else {
                i as u64 % 97
            }))
        })
        .collect();
    for value in &expected {
        source.push(value.unwrap());
    }
    source.write()?;
    let reader = source.read_only_clone();
    let published = expected.clone();
    assert_sorted_reads(&source, &expected);
    assert_sorted_reads(&reader, &published);
    let updates = [
        (0, TestValue(7)),
        (1, TestValue(9000)),
        (211, TestValue(11)),
    ];
    source.update_many(updates.to_vec())?;
    for (index, value) in updates {
        expected[index] = Some(value);
    }
    source.delete_many([2, 422]);
    for index in [2, 422] {
        expected[index] = None;
    }
    source.push(TestValue(77_777));
    expected.push(Some(TestValue(77_777)));
    assert_sorted_reads(&source, &expected);
    assert_sorted_reads(&reader, &published);
    source.write()?;
    assert_sorted_reads(&reader, &expected);
    source.truncate_if_needed_at(6000)?;
    expected.truncate(6000);
    for i in 0..100 {
        let value = TestValue(i);
        source.push(value);
        expected.push(Some(value));
    }
    source.write()?;
    assert_sorted_reads(&source, &expected);
    assert_sorted_reads(&reader, &expected);
    drop(source);
    let source = OverflowVec::<usize, TestValue>::import(&db, "sorted", Version::ONE)?;
    assert_sorted_reads(&source, &expected);
    assert_sorted_reads(&source.read_only_clone(), &expected);
    Ok(())
}

#[test]
fn rollback_and_truncation_keep_sidecar_in_sync() -> Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let options = ImportOptions::new(&db, "rollback", Version::ONE).with_saved_stamped_changes(5);
    let mut vec = OverflowVec::<usize, TestValue>::forced_import_with(options)?;

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
fn fill_holes_or_push_many_preserves_value_order_and_indexes() -> Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = OverflowVec::<usize, TestValue>::forced_import(&db, "insert", Version::ONE)?;

    for value in [1, 1_000, 2, 2_000] {
        vec.push(TestValue(value));
    }
    vec.write()?;
    vec.delete_many([1, 2]);
    vec.push(TestValue(3));
    vec.push(TestValue(3_000));
    vec.delete_many([4]);

    assert_eq!(
        vec.fill_holes_or_push_many(vec![
            TestValue(9),
            TestValue(9_000),
            TestValue(10),
            TestValue(10_000),
            TestValue(11),
        ]),
        vec![1, 2, 4, 6, 7]
    );
    assert_eq!(
        vec.collect(),
        vec![
            TestValue(1),
            TestValue(9),
            TestValue(9_000),
            TestValue(2_000),
            TestValue(10),
            TestValue(3_000),
            TestValue(10_000),
            TestValue(11),
        ]
    );
    assert!(vec.fill_holes_or_push_many(Vec::new()).is_empty());

    vec.write()?;
    drop(vec);
    let vec = OverflowVec::<usize, TestValue>::import(&db, "insert", Version::ONE)?;
    assert_eq!(
        vec.collect(),
        vec![
            TestValue(1),
            TestValue(9),
            TestValue(9_000),
            TestValue(2_000),
            TestValue(10),
            TestValue(3_000),
            TestValue(10_000),
            TestValue(11),
        ]
    );
    Ok(())
}

#[test]
fn sidecar_undo_is_prepared_before_either_half_is_overwritten() -> Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let options = ImportOptions::new(&db, "values", Version::ONE).with_saved_stamped_changes(4);
    let mut values = OverflowVec::<usize, TestValue>::import_with(options)?;
    values.push(TestValue(1000));
    values.stamped_write_with_changes(Stamp::new(1))?;
    let published = values.read_only_clone();
    fs::create_dir(temp.path().join("changes/values/usize/2"))?;
    values.update_many(vec![(0, TestValue(2000))])?;
    assert!(values.stamped_write_with_changes(Stamp::new(2)).is_err());
    assert_eq!(published.collect(), [TestValue(1000)]);
    assert_eq!(values.stamp(), Stamp::new(1));
    assert!(matches!(values.write(), Err(Error::WriteFailed)));
    assert!(matches!(values.reset(), Err(Error::WriteFailed)));
    assert!(matches!(
        values.update_many(vec![(0, TestValue(3))]),
        Err(Error::WriteFailed)
    ));
    Ok(())
}
