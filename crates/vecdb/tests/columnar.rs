use std::{
    ops::Add,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
    thread,
};

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, ColumnId, ColumnarVec, Database, EagerVec, Exit, ImportOptions,
    ImportableVec, LazyColumnSumVec, LazyColumnarVec, PrintableIndex, ReadableColumnarVec,
    ReadableVec, Result, Stamp, StoredVec, UnaryTransform, VecIndex, VecValue, Version,
    WritableVec,
};

const COLUMNS: usize = 3;
const U64S_PER_PAGE: usize = 16 * 1024 / size_of::<u64>();

macro_rules! column_ids {
    ($name:ident, $count:literal, $version:expr, [$($column:ident),+ $(,)?]) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(usize)]
        enum $name {
            $($column),+
        }

        impl ColumnId for $name {
            type Row<T>
                = [T; $count]
            where
                T: VecValue;

            const VERSION: Version = $version;
            const ALL: &'static [Self] = &[$(Self::$column),+];

            fn index(self) -> usize {
                self as usize
            }

            fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
                &row[self as usize]
            }

            fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
                &mut row[self as usize]
            }

            fn from_fn<T, F>(mut f: F) -> Self::Row<T>
            where
                T: VecValue,
                F: FnMut(Self) -> T,
            {
                std::array::from_fn(|index| f(Self::ALL[index]))
            }

            fn map<T, U, F>(row: Self::Row<T>, f: F) -> Self::Row<U>
            where
                T: VecValue,
                U: VecValue,
                F: FnMut(T) -> U,
            {
                row.map(f)
            }
        }
    };
}

column_ids!(TestColumn, 3, Version::ONE, [First, Second, Third]);
column_ids!(ChangedTestColumn, 3, Version::TWO, [First, Second, Third]);
column_ids!(
    FiveColumn,
    5,
    Version::ONE,
    [First, Second, Third, Fourth, Fifth]
);

struct Double;

impl UnaryTransform<u64> for Double {
    fn apply(value: u64) -> u64 {
        value * 2
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CapacityIndex(usize);

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

impl PrintableIndex for CapacityIndex {
    fn to_string() -> &'static str {
        "columnar_capacity"
    }

    fn to_possible_strings() -> &'static [&'static str] {
        &["columnar_capacity"]
    }
}

impl VecIndex for CapacityIndex {
    const INITIAL_CAPACITY: usize = 1_200_000;
}

fn row(index: usize) -> [u64; COLUMNS] {
    [
        index as u64,
        1_000_000 + index as u64 * 3,
        9_000_000 - index as u64 * 2,
    ]
}

#[test]
fn bytes_columnar_roundtrip_and_projection() -> Result<()> {
    type V = ColumnarVec<BytesVec<usize, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "matrix", Version::ONE)?;
    for index in 0..2_000 {
        vec.push(row(index));
    }
    vec.write()?;
    for index in 2_000..5_000 {
        vec.push(row(index));
    }

    assert_eq!(vec.collect_one_at(2_345), Some(row(2_345)));
    vec.write()?;
    assert_eq!(vec.region_names().len(), 1);

    let second = vec.column("second", Version::ONE, TestColumn::Second);
    assert!(second.region_names().is_empty());
    assert_eq!(second.collect_one_at(2_345), Some(row(2_345)[1]));
    assert_eq!(second.collect_range_at(4_990, 5_000).len(), 10);
    drop(second);
    drop(vec);

    let mut vec = V::import(&db, "matrix", Version::ONE)?;
    assert_eq!(vec.len(), 5_000);
    assert_eq!(vec.collect_one_at(0), Some(row(0)));
    assert_eq!(vec.collect_one_at(4_999), Some(row(4_999)));
    assert_eq!(
        vec.column("third", Version::ONE, TestColumn::Third)
            .collect_one_at(4_321),
        Some(row(4_321)[2])
    );

    vec.truncate_if_needed_at(2_503)?;
    for index in 2_503..2_777 {
        vec.push(row(index));
    }
    vec.write()?;
    drop(vec);

    let vec = V::import(&db, "matrix", Version::ONE)?;
    assert_eq!(vec.len(), 2_777);
    assert_eq!(vec.collect_one_at(2_502), Some(row(2_502)));
    assert_eq!(vec.collect_one_at(2_776), Some(row(2_776)));
    Ok(())
}

#[test]
fn eager_columnar_collect_last_tracks_persisted_and_pending_rows() -> Result<()> {
    type V = EagerVec<ColumnarVec<BytesVec<usize, u64>, TestColumn>>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "collect_last", Version::ONE)?;

    vec.push(row(1));
    vec.write()?;
    assert_eq!(vec.collect_last(), Some(row(1)));

    vec.push(row(2));
    assert_eq!(vec.collect_last(), Some(row(2)));

    Ok(())
}

#[test]
fn eager_columnar_computes_and_persists_rows() -> Result<()> {
    type Source = BytesVec<usize, u64>;
    type Target = EagerVec<ColumnarVec<BytesVec<usize, u64>, TestColumn>>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut source1 = Source::forced_import(&db, "eager_columnar_source_1", Version::ONE)?;
    let mut source2 = Source::forced_import(&db, "eager_columnar_source_2", Version::ONE)?;
    for value in 0..5_000_u64 {
        source1.push(value);
        source2.push(value + 10_000);
    }
    source1.write()?;
    source2.write()?;

    let mut target = Target::forced_import(&db, "eager_columnar_target", Version::ONE)?;
    target.compute_transform2_batched(
        0,
        &source1,
        &source2,
        777,
        |(index, value1, value2, ..)| (index, [value1, value2, value1 + value2]),
        &Exit::new(),
    )?;

    assert_eq!(target.len(), 5_000);
    assert_eq!(target.collect_one_at(4_321), Some([4_321, 14_321, 18_642]));
    assert_eq!(
        target
            .read_only_clone()
            .column("eager_columnar_second", Version::ONE, TestColumn::Second)
            .collect_one_at(4_321),
        Some(14_321)
    );
    drop(target);

    let target = Target::import(&db, "eager_columnar_target", Version::ONE)?;
    assert_eq!(target.collect_one_at(4_999), Some([4_999, 14_999, 19_998]));
    Ok(())
}

#[test]
fn projection_is_isolated_from_pushed_rows_until_write() -> Result<()> {
    type V = ColumnarVec<BytesVec<usize, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "projection_isolation", Version::ONE)?;
    for index in 0..100 {
        vec.push(row(index));
    }
    vec.write()?;
    let projection = vec.column("second", Version::ONE, TestColumn::Second);
    let sum = vec.sum_columns(
        "projection_isolation_sum",
        Version::ONE,
        [TestColumn::First, TestColumn::Second],
    );

    vec.push(row(100));
    assert_eq!(vec.len(), 101);
    assert_eq!(projection.len(), 100);
    assert_eq!(projection.collect_one_at(100), None);
    assert_eq!(sum.len(), 100);
    assert_eq!(sum.collect_one_at(100), None);

    vec.write()?;
    assert_eq!(projection.len(), 101);
    assert_eq!(projection.collect_one_at(100), Some(row(100)[1]));
    assert_eq!(sum.len(), 101);
    assert_eq!(sum.collect_one_at(100), Some(row(100)[0] + row(100)[1]));
    Ok(())
}

#[test]
fn lazy_columnar_transform_preserves_rows_and_columns() -> Result<()> {
    type V = ColumnarVec<BytesVec<usize, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "lazy", Version::ONE)?;
    for index in 0..5_000 {
        vec.push(row(index));
    }
    vec.write()?;

    let lazy = LazyColumnarVec::<_, u64, TestColumn>::transformed::<Double>(
        "doubled",
        Version::ONE,
        vec.read_only_clone(),
    );
    for index in [0, U64S_PER_PAGE - 1, U64S_PER_PAGE, 4_999] {
        assert_eq!(
            lazy.collect_one_at(index),
            Some(row(index).map(|value| value * 2))
        );
        assert_eq!(
            lazy.column("second", Version::ONE, TestColumn::Second)
                .collect_one_at(index),
            Some(row(index)[1] * 2)
        );
    }

    let from = U64S_PER_PAGE - 10;
    let to = U64S_PER_PAGE * 2 + 10;
    assert_eq!(
        lazy.collect_range_at(from, to),
        (from..to)
            .map(|index| row(index).map(|value| value * 2))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        lazy.column("second", Version::ONE, TestColumn::Second)
            .fold_range_at(from, to, 0, u64::wrapping_add),
        (from..to)
            .map(|index| row(index)[1] * 2)
            .fold(0, u64::wrapping_add)
    );
    Ok(())
}

#[test]
fn columnar_sum_accepts_stored_and_lazy_sources() -> Result<()> {
    type V = ColumnarVec<BytesVec<usize, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "sum", Version::ONE)?;
    for index in 0..5_000 {
        vec.push(row(index));
    }
    vec.write()?;

    let stored_sum = vec.sum_columns(
        "stored_sum",
        Version::ONE,
        [TestColumn::Third, TestColumn::First],
    );
    let source = vec.read_only_clone();
    let reordered_sum = LazyColumnSumVec::new(
        "reordered_sum",
        Version::ONE,
        source.clone(),
        [TestColumn::First, TestColumn::Third],
    );
    assert_eq!(stored_sum.version(), reordered_sum.version());
    let same_length_sum = LazyColumnSumVec::new(
        "same_length_sum",
        Version::ONE,
        source.clone(),
        [TestColumn::First, TestColumn::Second],
    );
    assert_eq!(stored_sum.version(), same_length_sum.version());
    let shorter_sum = LazyColumnSumVec::new(
        "shorter_sum",
        Version::ONE,
        source.clone(),
        [TestColumn::First],
    );
    assert_ne!(stored_sum.version(), shorter_sum.version());

    assert!(
        catch_unwind(AssertUnwindSafe(|| LazyColumnSumVec::new(
            "empty_sum",
            Version::ONE,
            source.clone(),
            [],
        )))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| LazyColumnSumVec::new(
            "duplicate_sum",
            Version::ONE,
            source.clone(),
            [TestColumn::First, TestColumn::First],
        )))
        .is_err()
    );

    let lazy = LazyColumnarVec::<_, u64, TestColumn>::transformed::<Double>(
        "doubled",
        Version::ONE,
        source,
    );
    let lazy_sum = lazy.sum_columns(
        "lazy_sum",
        Version::ONE,
        [TestColumn::First, TestColumn::Third],
    );

    for index in [0, U64S_PER_PAGE - 1, U64S_PER_PAGE, 4_999] {
        let expected = row(index)[0] + row(index)[2];
        assert_eq!(stored_sum.collect_one_at(index), Some(expected));
        assert_eq!(lazy_sum.collect_one_at(index), Some(expected * 2));
    }

    let from = U64S_PER_PAGE - 10;
    let to = U64S_PER_PAGE * 2 + 10;
    let expected = (from..to)
        .map(|index| row(index)[0] + row(index)[2])
        .collect::<Vec<_>>();
    assert_eq!(stored_sum.collect_range_at(from, to), expected);
    assert_eq!(
        lazy_sum.collect_range_at(from, to),
        expected
            .into_iter()
            .map(|value| value * 2)
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn raw_data_is_column_major_within_each_page_block() -> Result<()> {
    type V = ColumnarVec<BytesVec<usize, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "layout", Version::ONE)?;
    for index in 0..5_000 {
        vec.push(row(index));
    }
    vec.write()?;

    let bytes = vec.region().create_reader().read_all().to_vec();
    let stored = bytes[vecdb::HEADER_OFFSET..]
        .chunks_exact(size_of::<u64>())
        .map(|bytes| u64::from_le_bytes(bytes.try_into().unwrap()))
        .collect::<Vec<_>>();
    let mut expected = Vec::with_capacity(5_000 * COLUMNS);
    for block_start in (0..5_000).step_by(U64S_PER_PAGE) {
        let block_end = (block_start + U64S_PER_PAGE).min(5_000);
        for column in 0..COLUMNS {
            for index in block_start..block_end {
                expected.push(row(index)[column]);
            }
        }
    }
    assert_eq!(stored, expected);
    Ok(())
}

#[test]
fn column_count_is_part_of_storage_version() -> Result<()> {
    type ThreeColumns = ColumnarVec<BytesVec<usize, u64>, TestColumn>;
    type FiveColumns = ColumnarVec<BytesVec<usize, u64>, FiveColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = ThreeColumns::forced_import(&db, "column_count", Version::ONE)?;
    for index in 0..5 {
        vec.push(row(index));
    }
    vec.write()?;
    drop(vec);

    assert!(FiveColumns::import(&db, "column_count", Version::ONE).is_err());
    let vec = FiveColumns::forced_import(&db, "column_count", Version::ONE)?;
    assert!(vec.is_empty());
    Ok(())
}

#[test]
fn column_schema_version_is_part_of_storage_version() -> Result<()> {
    type Original = ColumnarVec<BytesVec<usize, u64>, TestColumn>;
    type Changed = ColumnarVec<BytesVec<usize, u64>, ChangedTestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = Original::forced_import(&db, "column_schema", Version::ONE)?;
    vec.push(row(0));
    vec.write()?;
    drop(vec);

    assert!(Changed::import(&db, "column_schema", Version::ONE).is_err());
    let vec = Changed::forced_import(&db, "column_schema", Version::ONE)?;
    assert!(vec.is_empty());
    Ok(())
}

#[test]
fn projected_try_fold_stops_at_the_first_error() -> Result<()> {
    type V = ColumnarVec<BytesVec<usize, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "try_fold", Version::ONE)?;
    for index in 0..5_000 {
        vec.push(row(index));
    }
    vec.write()?;

    let mut seen = 0;
    let result = vec
        .column("first", Version::ONE, TestColumn::First)
        .try_fold_range_at(0, 5_000, (), |(), _| -> std::result::Result<(), ()> {
            seen += 1;
            if seen == 17 { Err(()) } else { Ok(()) }
        });
    assert_eq!(result, Err(()));
    assert_eq!(seen, 17);
    Ok(())
}

#[test]
fn reset_and_rollback_persist() -> Result<()> {
    type V = ColumnarVec<BytesVec<usize, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let options = ImportOptions::new(&db, "changes", Version::ONE).with_saved_stamped_changes(3);
    let mut vec = V::forced_import_with(options)?;

    for index in 0..100 {
        vec.push(row(index));
    }
    vec.stamped_write_with_changes(Stamp::new(1))?;
    for index in 100..150 {
        vec.push(row(index));
    }
    vec.stamped_write_with_changes(Stamp::new(2))?;
    vec.rollback()?;
    assert_eq!(vec.len(), 100);
    assert_eq!(vec.collect_one_at(99), Some(row(99)));

    vec.stamped_write_with_changes(Stamp::new(2))?;
    vec.reset()?;
    vec.write()?;
    drop(vec);

    let vec = V::import_with(options)?;
    assert!(vec.is_empty());
    Ok(())
}

#[test]
fn initial_capacity_is_reserved_for_every_column() -> Result<()> {
    type V = ColumnarVec<BytesVec<CapacityIndex, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "capacity", Version::ONE)?;
    let expected =
        vecdb::HEADER_OFFSET + CapacityIndex::INITIAL_CAPACITY * COLUMNS * size_of::<u64>();
    assert!(vec.region().meta().reserved() >= expected);

    for index in 0..10_000 {
        vec.push(row(index));
    }
    vec.write()?;
    drop(vec);

    let vec = V::import(&db, "capacity", Version::ONE)?;
    assert_eq!(vec.len(), 10_000);
    assert_eq!(vec.collect_one_at(9_999), Some(row(9_999)));
    Ok(())
}

#[cfg(feature = "pco")]
#[test]
fn pco_columnar_roundtrip_reads_only_selected_stream() -> Result<()> {
    use vecdb::PcoVec;

    type V = ColumnarVec<PcoVec<usize, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "pco_matrix", Version::ONE)?;
    for index in 0..10_000 {
        vec.push(row(index));
    }
    vec.write()?;
    drop(vec);

    let vec = V::import(&db, "pco_matrix", Version::ONE)?;
    assert_eq!(vec.collect_one_at(9_999), Some(row(9_999)));
    assert_eq!(
        vec.column("second", Version::ONE, TestColumn::Second)
            .collect_range_at(9_990, 10_000),
        (9_990..10_000)
            .map(|index| row(index)[1])
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[cfg(feature = "pco")]
#[test]
fn pco_repeated_small_writes_compress_completed_pages_and_keep_tail_raw() -> Result<()> {
    use vecdb::PcoVec;

    type V = ColumnarVec<PcoVec<usize, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "pco_incremental", Version::ONE)?;
    for batch in 0..100 {
        for index in batch * 100..(batch + 1) * 100 {
            vec.push(row(index));
        }
        vec.write()?;
    }
    assert_eq!(vec.collect_one_at(9_999), Some(row(9_999)));

    let pages_region = db.get_region(&vec.region_names()[1]).expect("pages region");
    let bytes = pages_region.create_reader().read_all().to_vec();
    let pages = bytes
        .chunks_exact(16)
        .map(|bytes| u32::from_le_bytes(bytes[12..16].try_into().unwrap()))
        .collect::<Vec<_>>();
    // Completed column pages are compressed; only the final physical tail page
    // is required to remain raw while the logical row block is incomplete.
    let completed_pages = vec.len() / U64S_PER_PAGE * COLUMNS;
    for &values in &pages[..completed_pages] {
        assert_eq!(values & (1 << 31), 0);
        assert_eq!(values as usize, U64S_PER_PAGE);
    }
    assert!(pages.last().is_some_and(|values| values & (1 << 31) != 0));
    Ok(())
}

#[test]
fn concurrent_projection_reads_survive_incremental_writes() -> Result<()> {
    type V = ColumnarVec<BytesVec<usize, u64>, TestColumn>;

    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = V::forced_import(&db, "concurrent", Version::ONE)?;
    for index in 0..1_000 {
        vec.push(row(index));
    }
    vec.write()?;
    let projection = Arc::new(vec.column("third", Version::ONE, TestColumn::Third));
    let readers = (0..4)
        .map(|_| {
            let projection = Arc::clone(&projection);
            thread::spawn(move || {
                for _ in 0..2_000 {
                    let len = projection.len();
                    if len != 0 {
                        assert_eq!(projection.collect_one_at(len - 1), Some(row(len - 1)[2]));
                    }
                    thread::yield_now();
                }
            })
        })
        .collect::<Vec<_>>();

    for batch in 10..100 {
        for index in batch * 100..(batch + 1) * 100 {
            vec.push(row(index));
        }
        vec.write()?;
    }
    for reader in readers {
        reader.join().expect("reader thread");
    }
    assert_eq!(projection.len(), 10_000);
    Ok(())
}

#[cfg(feature = "lz4")]
#[test]
fn lz4_columnar_roundtrip() -> Result<()> {
    run_small_backend_roundtrip::<vecdb::LZ4Vec<usize, u64>>()
}

#[cfg(feature = "zstd")]
#[test]
fn zstd_columnar_roundtrip() -> Result<()> {
    run_small_backend_roundtrip::<vecdb::ZstdVec<usize, u64>>()
}

#[cfg(feature = "zerocopy")]
#[test]
fn zerocopy_columnar_roundtrip() -> Result<()> {
    run_small_backend_roundtrip::<vecdb::ZeroCopyVec<usize, u64>>()
}

fn run_small_backend_roundtrip<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u64> + 'static,
{
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = ColumnarVec::<V, TestColumn>::forced_import(&db, "backend", Version::ONE)?;
    for index in 0..4_200 {
        vec.push(row(index));
    }
    vec.write()?;
    drop(vec);

    let vec = ColumnarVec::<V, TestColumn>::import(&db, "backend", Version::ONE)?;
    assert_eq!(vec.collect_one_at(4_199), Some(row(4_199)));
    assert_eq!(
        vec.column("first", Version::ONE, TestColumn::First)
            .collect_one_at(3_123),
        Some(row(3_123)[0])
    );
    Ok(())
}
