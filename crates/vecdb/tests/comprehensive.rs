//! Comprehensive tests for RawVec functionality (take, holes, update, collect_holed).
//!
//! These tests are specific to BytesVec and ZeroCopyVec which implement ReadWriteRawVec.

use rawdb::Database;
use std::collections::BTreeSet;
use std::ops::DerefMut;
use tempfile::TempDir;
use vecdb::{ReadWriteRawVec, Stamp, StoredVec, VecReader, Version};

// ============================================================================
// Test Setup
// ============================================================================

fn setup_db() -> vecdb::Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

// ============================================================================
// Trait for Raw Vec Operations
// ============================================================================

pub trait RawVecOps {
    type Reader;

    fn take(&mut self, index: usize, reader: &Self::Reader) -> Option<u32>;
    fn update(&mut self, index: usize, value: u32) -> vecdb::Result<()>;
    fn holes(&self) -> &BTreeSet<usize>;
    fn collect_holed(&self) -> Vec<Option<u32>>;
    fn get_with_reader(&self, index: usize, reader: &Self::Reader) -> Option<u32>;
    fn reader(&self) -> Self::Reader;
}

impl<S> RawVecOps for ReadWriteRawVec<usize, u32, S>
where
    S: vecdb::RawStrategy<u32>,
{
    type Reader = VecReader<usize, u32, S>;

    fn take(&mut self, index: usize, reader: &Self::Reader) -> Option<u32> {
        ReadWriteRawVec::take(self, index, reader)
    }

    fn update(&mut self, index: usize, value: u32) -> vecdb::Result<()> {
        ReadWriteRawVec::update(self, index, value)
    }

    fn holes(&self) -> &BTreeSet<usize> {
        ReadWriteRawVec::holes(self)
    }

    fn collect_holed(&self) -> Vec<Option<u32>> {
        ReadWriteRawVec::collect_holed(self)
    }

    fn get_with_reader(&self, index: usize, reader: &Self::Reader) -> Option<u32> {
        ReadWriteRawVec::get_with_reader(self, index, reader)
    }

    fn reader(&self) -> Self::Reader {
        ReadWriteRawVec::reader(self)
    }
}

// ============================================================================
// Generic Comprehensive Tests
// ============================================================================

fn run_comprehensive_test<V>() -> vecdb::Result<()>
where
    V: StoredVec<I = usize, T = u32> + DerefMut,
    V::Target: RawVecOps,
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

        let reader = vec.deref_mut().reader();
        assert_eq!(vec.deref_mut().take(10, &reader), Some(10));
        assert_eq!(vec.deref_mut().holes(), &BTreeSet::from([10]));
        assert_eq!(vec.deref_mut().get_with_reader(10, &reader), None);
        drop(reader);

        vec.write()?;

        assert!(vec.deref_mut().holes() == &BTreeSet::from([10]));
    }

    {
        let mut vec = V::forced_import_with(options)?;

        assert!(vec.deref_mut().holes() == &BTreeSet::from([10]));

        let reader = vec.deref_mut().reader();
        assert!(vec.deref_mut().get_with_reader(10, &reader).is_none());
        drop(reader);

        vec.deref_mut().update(10, 10)?;
        vec.deref_mut().update(0, 10)?;

        let reader = vec.deref_mut().reader();
        assert_eq!(vec.deref_mut().holes(), &BTreeSet::new());
        assert_eq!(vec.deref_mut().get_with_reader(0, &reader), Some(10));
        assert_eq!(vec.deref_mut().get_with_reader(10, &reader), Some(10));
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

        let reader = vec.deref_mut().reader();
        let _ = vec.deref_mut().take(5, &reader);
        vec.deref_mut().update(3, 5)?;
        vec.push(21);
        drop(reader);

        assert_eq!(
            vec.deref_mut().collect_holed(),
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

        let reader = vec.deref_mut().reader();
        let _ = vec.deref_mut().take(0, &reader);
        vec.deref_mut().update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.deref_mut().collect_holed(),
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
            vec.deref_mut().collect_holed(),
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
            vec.deref_mut().collect_holed(),
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

        let reader = vec.deref_mut().reader();
        let _ = vec.deref_mut().take(5, &reader);
        vec.deref_mut().update(3, 5)?;
        vec.push(21);
        drop(reader);

        assert_eq!(
            vec.deref_mut().collect_holed(),
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

        let reader = vec.deref_mut().reader();
        let _ = vec.deref_mut().take(0, &reader);
        vec.deref_mut().update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.deref_mut().collect_holed(),
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
            vec.deref_mut().collect_holed(),
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
        let reader = vec.deref_mut().reader();
        let _ = vec.deref_mut().take(5, &reader);
        vec.deref_mut().update(3, 5)?;
        vec.push(21);
        drop(reader);

        let reader = vec.deref_mut().reader();
        let _ = vec.deref_mut().take(0, &reader);
        vec.deref_mut().update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.deref_mut().collect_holed(),
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
        let reader = vec.deref_mut().reader();
        let _ = vec.deref_mut().take(5, &reader);
        vec.deref_mut().update(3, 5)?;
        vec.push(21);
        drop(reader);

        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.stamp(), Stamp::new(1));

        let reader = vec.deref_mut().reader();
        let _ = vec.deref_mut().take(0, &reader);
        vec.deref_mut().update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        vec.stamped_write_with_changes(Stamp::new(2))?;

        assert_eq!(
            vec.deref_mut().collect_holed(),
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
        let reader = vec.deref_mut().reader();
        let _ = vec.deref_mut().take(5, &reader);
        vec.deref_mut().update(3, 5)?;
        vec.push(21);
        drop(reader);

        let reader = vec.deref_mut().reader();
        let _ = vec.deref_mut().take(0, &reader);
        vec.deref_mut().update(1, 5)?;
        vec.push(5);
        vec.push(6);
        vec.push(7);
        drop(reader);

        assert_eq!(
            vec.deref_mut().collect_holed(),
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
    use vecdb::BytesVec;
    type V = BytesVec<usize, u32>;

    #[test]
    fn test_raw_vec_comprehensive() -> vecdb::Result<()> {
        run_comprehensive_test::<V>()
    }
}

#[cfg(feature = "zerocopy")]
mod zerocopy {
    use super::*;
    use vecdb::ZeroCopyVec;
    type V = ZeroCopyVec<usize, u32>;

    #[test]
    fn test_raw_vec_comprehensive() -> vecdb::Result<()> {
        run_comprehensive_test::<V>()
    }
}
