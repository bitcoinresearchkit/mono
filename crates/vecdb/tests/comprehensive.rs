//! Comprehensive tests for mutable raw-vector functionality (take, holes, update, collect_holed).
//!
//! These tests exercise `MutableVec` over `BytesVec` and `ZeroCopyVec`.

use rawdb::Database;
use std::collections::BTreeSet;
use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, BytesVec, BytesVecReader, ImportableVec, MutableVec, ReadableVec, Stamp,
    StoredVec, Version, WritableVec,
};

#[cfg(feature = "zerocopy")]
use vecdb::{VecReader, ZeroCopyStrategy, ZeroCopyVec};

// ============================================================================
// Test Setup
// ============================================================================

fn setup_db() -> vecdb::Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

// ============================================================================
// Trait for Mutable Raw Vec Operations
// ============================================================================

pub trait RawVecOps: StoredVec<I = usize, T = u32> {
    type Reader;

    fn take(&mut self, index: usize, reader: &Self::Reader) -> Option<u32>;
    fn update(&mut self, index: usize, value: u32) -> vecdb::Result<()>;
    fn holes(&self) -> &BTreeSet<usize>;
    fn collect_holed(&self) -> Vec<Option<u32>>;
    fn get_with_reader(&self, index: usize, reader: &Self::Reader) -> Option<u32>;
    fn reader(&self) -> Self::Reader;
}

impl RawVecOps for MutableVec<BytesVec<usize, u32>> {
    type Reader = BytesVecReader<usize, u32>;

    fn take(&mut self, index: usize, reader: &Self::Reader) -> Option<u32> {
        MutableVec::<BytesVec<usize, u32>>::take(self, index, reader)
    }

    fn update(&mut self, index: usize, value: u32) -> vecdb::Result<()> {
        MutableVec::<BytesVec<usize, u32>>::update(self, index, value)
    }

    fn holes(&self) -> &BTreeSet<usize> {
        MutableVec::<BytesVec<usize, u32>>::holes(self)
    }

    fn collect_holed(&self) -> Vec<Option<u32>> {
        MutableVec::<BytesVec<usize, u32>>::collect_holed(self)
    }

    fn get_with_reader(&self, index: usize, reader: &Self::Reader) -> Option<u32> {
        MutableVec::<BytesVec<usize, u32>>::get_with_reader(self, index, reader)
    }

    fn reader(&self) -> Self::Reader {
        MutableVec::<BytesVec<usize, u32>>::reader(self)
    }
}

#[cfg(feature = "zerocopy")]
impl RawVecOps for MutableVec<ZeroCopyVec<usize, u32>> {
    type Reader = VecReader<usize, u32, ZeroCopyStrategy<u32>>;

    fn take(&mut self, index: usize, reader: &Self::Reader) -> Option<u32> {
        MutableVec::<ZeroCopyVec<usize, u32>>::take(self, index, reader)
    }

    fn update(&mut self, index: usize, value: u32) -> vecdb::Result<()> {
        MutableVec::<ZeroCopyVec<usize, u32>>::update(self, index, value)
    }

    fn holes(&self) -> &BTreeSet<usize> {
        MutableVec::<ZeroCopyVec<usize, u32>>::holes(self)
    }

    fn collect_holed(&self) -> Vec<Option<u32>> {
        MutableVec::<ZeroCopyVec<usize, u32>>::collect_holed(self)
    }

    fn get_with_reader(&self, index: usize, reader: &Self::Reader) -> Option<u32> {
        MutableVec::<ZeroCopyVec<usize, u32>>::get_with_reader(self, index, reader)
    }

    fn reader(&self) -> Self::Reader {
        MutableVec::<ZeroCopyVec<usize, u32>>::reader(self)
    }
}

// ============================================================================
// Generic Comprehensive Tests
// ============================================================================

fn run_comprehensive_test<V>() -> vecdb::Result<()>
where
    V: RawVecOps,
{
    let version = Version::TWO;
    let (database, _temp) = setup_db()?;
    let mut options = (&database, "vec", version).into();

    {
        let mut vec = V::forced_import_with(options)?;

        (0..21_u32).for_each(|v| {
            vec.push(v);
        });

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(1, 2), vec![1]);
        assert_eq!(vec.collect_range(2, 3), vec![2]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert!(vec.collect_range(21, 22).is_empty());

        vec.write()?;

        assert!(vec.header().stamp() == Stamp::new(0));
    }

    {
        let mut vec = V::forced_import_with(options)?;

        vec.mut_header().update_stamp(Stamp::new(100));

        assert_eq!(vec.header().stamp(), Stamp::new(100));

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(1, 2), vec![1]);
        assert_eq!(vec.collect_range(2, 3), vec![2]);
        assert_eq!(vec.collect_range(3, 4), vec![3]);
        assert_eq!(vec.collect_range(4, 5), vec![4]);
        assert_eq!(vec.collect_range(5, 6), vec![5]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert_eq!(vec.collect_range(0, 1), vec![0]);

        vec.push(21);
        vec.push(22);

        assert_eq!(vec.stored_len(), 21);
        assert_eq!(vec.pushed_len(), 2);
        assert_eq!(vec.len(), 23);

        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert_eq!(vec.collect_range(21, 22), vec![21]);
        assert_eq!(vec.collect_range(22, 23), vec![22]);
        assert!(vec.collect_range(23, 24).is_empty());

        vec.write()?;
    }

    {
        let mut vec = V::forced_import_with(options)?;

        assert_eq!(vec.header().stamp(), Stamp::new(100));

        assert_eq!(vec.stored_len(), 23);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.len(), 23);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert_eq!(vec.collect_range(21, 22), vec![21]);
        assert_eq!(vec.collect_range(22, 23), vec![22]);

        vec.truncate_if_needed(14)?;

        assert_eq!(vec.stored_len(), 14);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.len(), 14);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(5, 6), vec![5]);
        assert!(vec.collect_range(20, 21).is_empty());

        assert_eq!(
            vec.collect_signed_range(Some(-5), None),
            vec![9, 10, 11, 12, 13]
        );

        vec.push(vec.len() as u32);
        assert_eq!(vec.collect_range(vec.len() - 1, vec.len())[0], 14);

        assert_eq!(
            vec.collect(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );

        vec.write()?;
    }

    {
        let mut vec = V::forced_import_with(options)?;

        assert_eq!(vec.collect_range(vec.len() - 1, vec.len())[0], 14);

        assert_eq!(
            vec.collect(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );

        vec.reset()?;

        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.len(), 0);

        (0..21_u32).for_each(|v| {
            vec.push(v);
        });

        assert_eq!(vec.pushed_len(), 21);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.len(), 21);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert!(vec.collect_range(21, 22).is_empty());

        let reader = vec.reader();
        assert_eq!(vec.take(10, &reader), Some(10));
        assert_eq!(vec.holes(), &BTreeSet::from([10]));
        assert_eq!(vec.get_with_reader(10, &reader), None);
        drop(reader);

        vec.write()?;

        assert!(vec.holes() == &BTreeSet::from([10]));
    }

    {
        let mut vec = V::forced_import_with(options)?;

        assert!(vec.holes() == &BTreeSet::from([10]));

        let reader = vec.reader();
        assert!(vec.get_with_reader(10, &reader).is_none());
        drop(reader);

        vec.update(10, 10)?;
        vec.update(0, 10)?;

        let reader = vec.reader();
        assert_eq!(vec.holes(), &BTreeSet::new());
        assert_eq!(vec.get_with_reader(0, &reader), Some(10));
        assert_eq!(vec.get_with_reader(10, &reader), Some(10));
        drop(reader);

        vec.write()?;
    }

    options = options.with_saved_stamped_changes(10);

    {
        let mut vec = V::forced_import_with(options)?;

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.truncate_if_needed(10)?;

        let reader = vec.reader();
        let _ = vec.take(5, &reader);
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        assert_eq!(
            vec.collect_holed(),
            vec![
                Some(10),
                Some(1),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21)
            ]
        );

        vec.stamped_write_with_changes(Stamp::new(1))?;
    }

    {
        let mut vec = V::forced_import_with(options)?;

        assert_eq!(vec.collect(), vec![10, 1, 2, 5, 4, 6, 7, 8, 9, 21]);

        let reader = vec.reader();
        let _ = vec.take(0, &reader);
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        vec.stamped_write_with_changes(Stamp::new(2))?;
    }

    {
        let mut vec = V::forced_import_with(options)?;

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        vec.rollback()?;

        assert_eq!(vec.stamp(), Stamp::new(1));

        assert_eq!(
            vec.collect_holed(),
            vec![
                Some(10),
                Some(1),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21)
            ]
        );

        vec.rollback()?;

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.stamped_write(Stamp::new(0))?;
    }

    {
        let mut vec = V::forced_import_with(options)?;

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.truncate_if_needed(10)?;

        let reader = vec.reader();
        let _ = vec.take(5, &reader);
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        assert_eq!(
            vec.collect_holed(),
            vec![
                Some(10),
                Some(1),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21)
            ]
        );

        vec.stamped_write_with_changes(Stamp::new(1))?;
    }

    {
        let mut vec = V::forced_import_with(options)?;

        assert_eq!(vec.collect(), vec![10, 1, 2, 5, 4, 6, 7, 8, 9, 21]);

        let reader = vec.reader();
        let _ = vec.take(0, &reader);
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        vec.stamped_write_with_changes(Stamp::new(2))?;
    }

    {
        let mut vec = V::forced_import_with(options)?;

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        let _ = vec.rollback_before(Stamp::new(1))?;

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.stamped_write(Stamp::new(0))?;

        vec.truncate_if_needed(10)?;
        let reader = vec.reader();
        let _ = vec.take(5, &reader);
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        let reader = vec.reader();
        let _ = vec.take(0, &reader);
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );
    }

    {
        let mut vec = V::forced_import_with(options)?;

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.truncate_if_needed(10)?;
        let reader = vec.reader();
        let _ = vec.take(5, &reader);
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.stamp(), Stamp::new(1));

        let reader = vec.reader();
        let _ = vec.take(0, &reader);
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        vec.stamped_write_with_changes(Stamp::new(2))?;

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        let _ = vec.rollback_before(Stamp::new(1))?;

        assert_eq!(vec.stamp(), Stamp::new(0));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.truncate_if_needed(10)?;
        let reader = vec.reader();
        let _ = vec.take(5, &reader);
        vec.update(3, 5)?;
        vec.push(21);
        drop(reader);

        let reader = vec.reader();
        let _ = vec.take(0, &reader);
        vec.update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.collect_holed(),
            vec![
                None,
                Some(5),
                Some(2),
                Some(5),
                Some(4),
                None,
                Some(6),
                Some(7),
                Some(8),
                Some(9),
                Some(21),
                Some(5),
                Some(6),
                Some(7)
            ]
        );

        assert_eq!(vec.stamp(), Stamp::new(0));
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.stamp(), Stamp::new(2));

        let _ = vec.rollback_before(Stamp::new(1))?;

        assert_eq!(vec.stamp(), Stamp::new(0));

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );

        vec.stamped_write_with_changes(Stamp::new(0))?;

        let vec = V::forced_import_with(options)?;

        assert_eq!(
            vec.collect(),
            vec![
                10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );
    }

    Ok(())
}

// ============================================================================
// Test instantiation for BytesVec and ZeroCopyVec
// ============================================================================

mod bytes {
    use super::*;

    #[test]
    fn test_raw_vec_comprehensive() -> vecdb::Result<()> {
        run_comprehensive_test::<MutableVec<BytesVec<usize, u32>>>()
    }
}

#[cfg(feature = "zerocopy")]
mod zerocopy {
    use super::*;

    #[test]
    fn test_raw_vec_comprehensive() -> vecdb::Result<()> {
        run_comprehensive_test::<MutableVec<ZeroCopyVec<usize, u32>>>()
    }
}

#[test]
fn read_only_clone_tracks_published_holes() -> vecdb::Result<()> {
    let (database, _temp) = setup_db()?;
    let mut vec = MutableVec::<BytesVec<usize, u32>>::forced_import(
        &database,
        "read_only_holes",
        Version::ONE,
    )?;
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.write()?;

    let read_only = vec.read_only_clone();
    vec.delete_at(1);
    vec.write()?;

    assert_eq!(read_only.collect(), vec![1, 3]);
    assert_eq!(read_only.collect_one_at(1), None);

    vec.update_at(1, 4)?;
    vec.write()?;
    assert_eq!(read_only.collect(), vec![1, 4, 3]);

    Ok(())
}
