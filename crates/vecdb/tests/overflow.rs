use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Bytes, Database, ImportOptions, ImportableVec, OverflowVec,
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
fn large_range_decodes_inline_and_overflow_values() -> Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = OverflowVec::<usize, TestValue>::forced_import(&db, "large", Version::ONE)?;
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
    let db = Database::open(temp.path())?;
    let mut vec = OverflowVec::<usize, TestValue>::forced_import(&db, "values", Version::ONE)?;
    let read_only = vec.read_only_clone();

    vec.push(TestValue(7));
    vec.push(TestValue(1_000));
    vec.push(TestValue(3));
    let reader = vec.reader();
    assert_eq!(vec.get_with_reader(1, &reader), Some(TestValue(1_000)));
    assert_eq!(read_only.len(), 0);
    drop(reader);

    vec.write()?;

    let mut cursor = vec.reader().cursor();
    cursor.advance(1);
    assert_eq!(cursor.next(), Some(TestValue(1_000)));
    assert_eq!(cursor.position(), 2);
    assert_eq!(cursor.remaining(), 1);
    assert_eq!(cursor.get(2), Some(TestValue(3)));
    drop(cursor);

    assert_eq!(
        read_only.collect(),
        vec![TestValue(7), TestValue(1_000), TestValue(3)]
    );
    assert_eq!(vec.region_names().len(), 2);

    vec.update_many(vec![(0, TestValue(5_000)), (1, TestValue(6_000))])?;
    vec.delete_many([1, 0]);
    assert_eq!(vec.holes().len(), 2);
    assert_eq!(
        vec.fill_holes_or_push_many(vec![TestValue(9), TestValue(7_000)]),
        vec![0, 1]
    );
    assert_eq!(
        vec.collect(),
        vec![TestValue(9), TestValue(7_000), TestValue(3)]
    );

    vec.write()?;
    drop(vec);

    let vec = OverflowVec::<usize, TestValue>::import(&db, "values", Version::ONE)?;
    assert_eq!(
        vec.collect(),
        vec![TestValue(9), TestValue(7_000), TestValue(3)]
    );
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
fn forced_version_reset_removes_data_and_holes() -> Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = OverflowVec::<usize, TestValue>::forced_import(&db, "reset", Version::ONE)?;
    vec.push(TestValue(1_000));
    vec.push(TestValue(2_000));
    vec.write()?;
    vec.delete_many([0]);
    vec.write()?;
    assert_eq!(vec.holes().len(), 1);
    drop(vec);

    let mut vec = OverflowVec::<usize, TestValue>::forced_import(&db, "reset", Version::TWO)?;
    assert!(vec.is_empty());
    assert!(vec.holes().is_empty());
    assert_eq!(vec.fill_holes_or_push_many(vec![TestValue(3_000)]), vec![0]);
    vec.write()?;
    assert_eq!(vec.collect(), vec![TestValue(3_000)]);
    Ok(())
}

#[test]
fn fills_holes_in_unwritten_values() -> Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = OverflowVec::<usize, TestValue>::forced_import(&db, "pushed", Version::ONE)?;

    vec.push(TestValue(1));
    vec.push(TestValue(2));
    vec.push(TestValue(3));
    vec.delete_many([1]);

    assert_eq!(vec.fill_holes_or_push_many(vec![TestValue(4)]), vec![1]);
    assert!(vec.holes().is_empty());
    assert_eq!(
        vec.collect(),
        vec![TestValue(1), TestValue(4), TestValue(3)]
    );
    Ok(())
}

#[test]
fn update_many_batches_final_values_across_every_storage_state() -> Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = OverflowVec::<usize, TestValue>::forced_import(&db, "batch", Version::ONE)?;

    for value in [1, 1_000, 2, 2_000] {
        vec.push(TestValue(value));
    }
    vec.write()?;
    vec.delete_many([2]);
    vec.push(TestValue(3));
    vec.push(TestValue(3_000));

    vec.update_many(vec![
        (5, TestValue(6)),
        (2, TestValue(6_000)),
        (1, TestValue(5)),
        (4, TestValue(7_000)),
        (0, TestValue(7)),
    ])?;
    assert_eq!(
        vec.collect(),
        vec![
            TestValue(7),
            TestValue(5),
            TestValue(6_000),
            TestValue(2_000),
            TestValue(7_000),
            TestValue(6),
        ]
    );

    assert!(
        vec.update_many(vec![(0, TestValue(8)), (vec.len(), TestValue(9))])
            .is_err()
    );
    assert_eq!(vec.collect_one(0), Some(TestValue(7)));
    vec.write()?;
    drop(vec);
    let vec = OverflowVec::<usize, TestValue>::import(&db, "batch", Version::ONE)?;
    assert_eq!(
        vec.collect(),
        vec![
            TestValue(7),
            TestValue(5),
            TestValue(6_000),
            TestValue(2_000),
            TestValue(7_000),
            TestValue(6),
        ]
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
