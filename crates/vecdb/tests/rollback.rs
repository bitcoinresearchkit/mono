//! Generic rollback tests for all vec types.
//!
//! This module contains two sets of rollback tests:
//! 1. Generic rollback tests - work with ALL vec types (BytesVec, ZeroCopyVec, PcoVec, LZ4Vec, ZstdVec, EagerVec)
//!    These use only push/truncate operations available on all vecs.
//! 2. Mutable raw-vector rollback tests - work with `MutableVec<BytesVec>` and
//!    `MutableVec<ZeroCopyVec>` only. These test update and hole operations.

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, ImportOptions, ImportableVec, Result, Stamp, StoredVec, Version, WritableVec,
};

// Test Setup

fn setup_db() -> Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

// PART 1: Generic Rollback Tests (ALL vec types)

// These tests use only push/truncate operations and work with any StoredVec.

mod generic_rollback {
    use vecdb::BytesVec;
    #[cfg(any(feature = "zerocopy", feature = "pco"))]
    use vecdb::EagerVec;
    #[cfg(feature = "lz4")]
    use vecdb::LZ4Vec;
    #[cfg(feature = "pco")]
    use vecdb::PcoVec;
    #[cfg(feature = "zerocopy")]
    use vecdb::ZeroCopyVec;
    #[cfg(feature = "zstd")]
    use vecdb::ZstdVec;

    use super::*;

    fn import_with_changes<V>(db: &Database, name: &str, changes: u16) -> Result<V>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let mut options: ImportOptions = (db, name, Version::TWO).into();
        options = options.with_saved_stamped_changes(changes);
        V::forced_import_with(options)
    }

    fn run_basic_rollback<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<V>(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4]
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        // Stamp 2: [0, 1, 2, 3, 4, 5, 6]
        let undo = vec.serialize_changes()?;
        vec.push(5);
        vec.push(6);
        assert_eq!(vec.serialize_changes()?, undo);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(vec.stamp(), Stamp::new(2));

        // Rollback to stamp 1
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        Ok(())
    }

    fn run_rollback_then_save_new_state<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<V>(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4]
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: [0, 1, 2, 3, 4, 5]
        vec.push(5);
        vec.stamped_write_with_changes(Stamp::new(2))?;

        // Rollback to stamp 1
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);

        // Save new stamp 2: [0, 1, 2, 3, 4, 99]
        vec.push(99);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 99]);
        assert_eq!(vec.stamp(), Stamp::new(2));

        Ok(())
    }

    fn run_rollback_to_empty<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<V>(&db, "test", 10)?;

        // Stamp 1: []
        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.collect(), Vec::<u32>::new());

        // Stamp 2: [0, 1, 2]
        vec.push(0);
        vec.push(1);
        vec.push(2);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 1, 2]);

        // Rollback to empty
        vec.rollback()?;
        assert_eq!(vec.collect(), Vec::<u32>::new());

        Ok(())
    }

    fn run_rollback_before<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<V>(&db, "test", 10)?;

        // Build stamps 1-5
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        vec.push(5);
        vec.stamped_write_with_changes(Stamp::new(2))?;

        vec.push(6);
        vec.stamped_write_with_changes(Stamp::new(3))?;

        vec.push(7);
        vec.stamped_write_with_changes(Stamp::new(4))?;

        vec.push(8);
        vec.stamped_write_with_changes(Stamp::new(5))?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);

        // Rollback before stamp 4 (should go to stamp 3)
        let _ = vec.rollback_before(Stamp::new(4))?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(vec.stamp(), Stamp::new(3));

        Ok(())
    }

    fn run_deep_rollback_chain<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<V>(&db, "test", 10)?;

        // Build chain of stamps with pushes only
        vec.stamped_write_with_changes(Stamp::new(1))?; // []

        vec.push(0);
        vec.stamped_write_with_changes(Stamp::new(2))?; // [0]

        vec.push(1);
        vec.stamped_write_with_changes(Stamp::new(3))?; // [0, 1]

        vec.push(2);
        vec.stamped_write_with_changes(Stamp::new(4))?; // [0, 1, 2]

        vec.push(3);
        vec.push(4);
        vec.stamped_write_with_changes(Stamp::new(5))?; // [0, 1, 2, 3, 4]
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);

        // Rollback through chain
        vec.rollback()?; // -> 4
        assert_eq!(vec.collect(), vec![0, 1, 2]);

        vec.rollback()?; // -> 3
        assert_eq!(vec.collect(), vec![0, 1]);

        vec.rollback()?; // -> 2
        assert_eq!(vec.collect(), vec![0]);

        vec.rollback()?; // -> 1
        assert_eq!(vec.collect(), Vec::<u32>::new());

        Ok(())
    }

    fn run_rollback_persistence<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;

        // Create and populate
        {
            let mut vec = import_with_changes::<V>(&db, "test", 10)?;

            for i in 0..5 {
                vec.push(i);
            }
            vec.stamped_write_with_changes(Stamp::new(1))?;

            vec.push(5);
            vec.push(6);
            vec.stamped_write_with_changes(Stamp::new(2))?;

            // Rollback and flush
            vec.rollback()?;
            vec.stamped_write_with_changes(Stamp::new(1))?;
        }

        // Reopen and verify
        {
            let vec = import_with_changes::<V>(&db, "test", 10)?;
            assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);
            assert_eq!(vec.stamp(), Stamp::new(1));
        }

        Ok(())
    }

    fn run_reset<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<V>(&db, "test", 10)?;

        // Add initial data and flush
        for i in 0..10 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.len(), 10);
        assert_eq!(vec.stored_len(), 10);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        // Add more data without flushing
        vec.push(10);
        vec.push(11);
        assert_eq!(vec.len(), 12);
        assert_eq!(vec.stored_len(), 10);
        assert_eq!(vec.pushed_len(), 2);

        // Reset should clear everything
        vec.reset()?;
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.collect(), Vec::<u32>::new());

        // Should be able to add new data after reset
        vec.push(100);
        vec.push(101);
        vec.push(102);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.pushed_len(), 3);
        assert_eq!(vec.collect(), vec![100, 101, 102]);

        // Flush the new data
        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.stored_len(), 3);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.collect(), vec![100, 101, 102]);

        Ok(())
    }

    // Test modules for each vec type
    fn run<V: StoredVec<I = usize, T = u32>>() -> Result<()> {
        run_basic_rollback::<V>()?;
        run_rollback_then_save_new_state::<V>()?;
        run_rollback_to_empty::<V>()?;
        run_rollback_before::<V>()?;
        run_deep_rollback_chain::<V>()?;
        run_rollback_persistence::<V>()?;
        run_reset::<V>()?;
        Ok(())
    }

    #[test]
    fn bytes() -> Result<()> {
        run::<BytesVec<usize, u32>>()
    }

    #[cfg(feature = "zerocopy")]
    #[test]
    fn zerocopy() -> Result<()> {
        run::<ZeroCopyVec<usize, u32>>()
    }

    #[cfg(feature = "pco")]
    #[test]
    fn pco() -> Result<()> {
        run::<PcoVec<usize, u32>>()
    }

    #[cfg(feature = "lz4")]
    #[test]
    fn lz4() -> Result<()> {
        run::<LZ4Vec<usize, u32>>()
    }

    #[cfg(feature = "zstd")]
    #[test]
    fn zstd() -> Result<()> {
        run::<ZstdVec<usize, u32>>()
    }

    #[cfg(feature = "zerocopy")]
    #[test]
    fn eager_zerocopy() -> Result<()> {
        run::<EagerVec<ZeroCopyVec<usize, u32>>>()
    }

    #[cfg(feature = "pco")]
    #[test]
    fn eager_pco() -> Result<()> {
        run::<EagerVec<PcoVec<usize, u32>>>()
    }
}

// PART 2: Raw-Only Rollback Tests (BytesVec and ZeroCopyVec)

// These tests use update/hole operations provided by MutableVec over raw vecs.

mod raw_rollback {
    use vecdb::{BytesVec, BytesVecReader, MutableVec};

    use super::*;

    // ============================================================================
    // Trait for mutable raw-vector rollback operations
    // ============================================================================

    /// Trait for mutable raw vecs that support rollback operations.
    pub trait RollbackVec: StoredVec<I = usize, T = u32> + RollbackOps {
        fn import_with_changes<'a>(
            db: &'a Database,
            name: &'a str,
            changes: u16,
        ) -> Result<(Self, ImportOptions<'a>)>;
    }

    /// Operations required for rollback testing.
    pub trait RollbackOps {
        type Reader;

        fn update(&mut self, index: usize, value: u32) -> Result<()>;
        fn take(&mut self, index: usize) -> Option<u32>;
        fn collect_holed(&self) -> Vec<Option<u32>>;
        fn get_with_reader(&self, index: usize, reader: &Self::Reader) -> Option<u32>;
        fn reader(&self) -> Self::Reader;
    }

    // ============================================================================
    // Implementations for ZeroCopyVec
    // ============================================================================

    #[cfg(feature = "zerocopy")]
    use vecdb::{VecReader, ZeroCopyStrategy, ZeroCopyVec};

    #[cfg(feature = "zerocopy")]
    impl RollbackVec for MutableVec<ZeroCopyVec<usize, u32>> {
        fn import_with_changes<'a>(
            db: &'a Database,
            name: &'a str,
            changes: u16,
        ) -> Result<(Self, ImportOptions<'a>)> {
            let mut options: ImportOptions = (db, name, Version::TWO).into();
            options = options.with_saved_stamped_changes(changes);
            let vec = Self::forced_import_with(options)?;
            Ok((vec, options))
        }
    }

    #[cfg(feature = "zerocopy")]
    impl RollbackOps for MutableVec<ZeroCopyVec<usize, u32>> {
        type Reader = VecReader<usize, u32, ZeroCopyStrategy<u32>>;

        fn update(&mut self, index: usize, value: u32) -> Result<()> {
            MutableVec::<ZeroCopyVec<usize, u32>>::update(self, index, value)
        }

        fn take(&mut self, index: usize) -> Option<u32> {
            let reader = MutableVec::<ZeroCopyVec<usize, u32>>::reader(self);
            MutableVec::<ZeroCopyVec<usize, u32>>::take(self, index, &reader)
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
    // Implementations for BytesVec
    // ============================================================================

    impl RollbackVec for MutableVec<BytesVec<usize, u32>> {
        fn import_with_changes<'a>(
            db: &'a Database,
            name: &'a str,
            changes: u16,
        ) -> Result<(Self, ImportOptions<'a>)> {
            let mut options: ImportOptions = (db, name, Version::TWO).into();
            options = options.with_saved_stamped_changes(changes);
            let vec = Self::forced_import_with(options)?;
            Ok((vec, options))
        }
    }

    impl RollbackOps for MutableVec<BytesVec<usize, u32>> {
        type Reader = BytesVecReader<usize, u32>;

        fn update(&mut self, index: usize, value: u32) -> Result<()> {
            MutableVec::<BytesVec<usize, u32>>::update(self, index, value)
        }

        fn take(&mut self, index: usize) -> Option<u32> {
            let reader = MutableVec::<BytesVec<usize, u32>>::reader(self);
            MutableVec::<BytesVec<usize, u32>>::take(self, index, &reader)
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

    // ============================================================================
    // Generic Rollback Test Functions
    // ============================================================================

    fn run_rollback_then_save_new_state<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4]
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: [0, 1, 2, 3, 4, 5]
        vec.push(5);
        vec.stamped_write_with_changes(Stamp::new(2))?;

        // Rollback to stamp 1
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);

        // Now save a different state 2: [0, 1, 2, 3, 4, 99]
        vec.push(99);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 99]);
        assert_eq!(vec.stamp(), Stamp::new(2));

        Ok(())
    }

    fn run_rollback_with_updates<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4]
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: [0, 99, 2, 88, 4] - update multiple values
        vec.update(1, 99)?;
        vec.update(3, 88)?;
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 99, 2, 88, 4]);

        // Rollback to stamp 1 - should restore original values
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        Ok(())
    }

    fn run_rollback_with_holes<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4]
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: delete some items (creating holes)
        let _ = vec.take(1);
        let _ = vec.take(3);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 2, 4]);

        // Rollback to stamp 1 - should restore deleted items
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        Ok(())
    }

    fn run_rollback_with_truncation_and_updates<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4]
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: extend + update
        vec.update(1, 99)?;
        vec.push(5);
        vec.push(6);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 99, 2, 3, 4, 5, 6]);

        // Rollback - should restore length AND value
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        Ok(())
    }

    fn run_rollback_with_holes_and_updates<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4]
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: delete + update
        let _ = vec.take(1);
        vec.update(2, 99)?;
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 99, 3, 4]);

        // Rollback - should restore deleted item AND original value
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        Ok(())
    }

    fn run_multiple_updates_to_same_index<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4]
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: [100, 1, 2, 3, 4]
        vec.update(0, 100)?;
        vec.stamped_write_with_changes(Stamp::new(2))?;

        // Stamp 3: [200, 1, 2, 3, 4]
        vec.update(0, 200)?;
        vec.stamped_write_with_changes(Stamp::new(3))?;

        // Stamp 4: [300, 1, 2, 3, 4]
        vec.update(0, 300)?;
        vec.stamped_write_with_changes(Stamp::new(4))?;
        assert_eq!(vec.collect(), vec![300, 1, 2, 3, 4]);

        // Rollback to stamp 3
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![200, 1, 2, 3, 4]);

        // Rollback to stamp 2
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![100, 1, 2, 3, 4]);

        // Rollback to stamp 1
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);

        Ok(())
    }

    fn run_complex_mixed_operations<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        for i in 0..10 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: Complex operations
        // - Delete indices 1, 3, 5
        // - Update indices 2, 6, 8
        // - Push new values 100, 101
        let _ = vec.take(1);
        let _ = vec.take(3);
        let _ = vec.take(5);
        vec.update(2, 222)?;
        vec.update(6, 666)?;
        vec.update(8, 888)?;
        vec.push(100);
        vec.push(101);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 222, 4, 666, 7, 888, 9, 100, 101]);

        // Rollback - should restore everything
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        Ok(())
    }

    fn run_rollback_to_empty<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1: []
        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.collect(), Vec::<u32>::new());

        // Stamp 2: [0, 1, 2]
        vec.push(0);
        vec.push(1);
        vec.push(2);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 1, 2]);

        // Rollback to empty
        vec.rollback()?;
        assert_eq!(vec.collect(), Vec::<u32>::new());

        Ok(())
    }

    fn run_reset<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Add initial data and flush
        for i in 0..10 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.len(), 10);
        assert_eq!(vec.stored_len(), 10);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        // Add more data without flushing
        vec.push(10);
        vec.push(11);
        assert_eq!(vec.len(), 12);
        assert_eq!(vec.stored_len(), 10);
        assert_eq!(vec.pushed_len(), 2);

        // Reset should clear everything
        vec.reset()?;
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.collect(), Vec::<u32>::new());

        // Should be able to add new data after reset
        vec.push(100);
        vec.push(101);
        vec.push(102);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.pushed_len(), 3);
        assert_eq!(vec.collect(), vec![100, 101, 102]);

        // Flush the new data
        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.stored_len(), 3);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.collect(), vec![100, 101, 102]);

        Ok(())
    }

    fn run_deep_rollback_chain<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Build a chain of 10 stamps with different operations
        vec.stamped_write_with_changes(Stamp::new(1))?; // []

        vec.push(0);
        vec.stamped_write_with_changes(Stamp::new(2))?; // [0]

        vec.push(1);
        vec.stamped_write_with_changes(Stamp::new(3))?; // [0, 1]

        vec.update(0, 10)?;
        vec.stamped_write_with_changes(Stamp::new(4))?; // [10, 1]

        vec.push(2);
        vec.stamped_write_with_changes(Stamp::new(5))?; // [10, 1, 2]

        let _ = vec.take(1);
        vec.stamped_write_with_changes(Stamp::new(6))?; // [10, 2]

        vec.push(3);
        vec.stamped_write_with_changes(Stamp::new(7))?; // [10, 2, 3]

        vec.update(0, 20)?;
        vec.stamped_write_with_changes(Stamp::new(8))?; // [20, 2, 3]

        vec.push(4);
        vec.push(5);
        vec.stamped_write_with_changes(Stamp::new(9))?; // [20, 2, 3, 4, 5]

        vec.update(2, 33)?;
        vec.stamped_write_with_changes(Stamp::new(10))?; // [20, 33, 3, 4, 5]
        assert_eq!(vec.collect(), vec![20, 33, 3, 4, 5]);

        // Rollback through the chain
        vec.rollback()?; // -> 9
        assert_eq!(vec.collect(), vec![20, 2, 3, 4, 5]);

        vec.rollback()?; // -> 8
        assert_eq!(vec.collect(), vec![20, 2, 3]);

        vec.rollback()?; // -> 7
        assert_eq!(vec.collect(), vec![10, 2, 3]);

        vec.rollback()?; // -> 6
        assert_eq!(vec.collect(), vec![10, 2]);

        vec.rollback()?; // -> 5
        assert_eq!(vec.collect(), vec![10, 1, 2]);

        vec.rollback()?; // -> 4
        assert_eq!(vec.collect(), vec![10, 1]);

        vec.rollback()?; // -> 3
        assert_eq!(vec.collect(), vec![0, 1]);

        vec.rollback()?; // -> 2
        assert_eq!(vec.collect(), vec![0]);

        vec.rollback()?; // -> 1
        assert_eq!(vec.collect(), Vec::<u32>::new());

        Ok(())
    }

    fn run_rollback_all_elements_updated<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4]
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: Update ALL elements
        for i in 0..5 {
            vec.update(i, (i * 100) as u32)?;
        }
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![0, 100, 200, 300, 400]);

        // Rollback - should restore all original values
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);

        Ok(())
    }

    fn run_multiple_holes_then_rollback<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        for i in 0..10 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: Delete every other element
        for i in (0..10).step_by(2) {
            let _ = vec.take(i);
        }
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![1, 3, 5, 7, 9]);

        // Rollback - should restore all deleted items
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        Ok(())
    }

    fn run_rollback_before<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Build stamps 1-5
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        vec.push(5);
        vec.stamped_write_with_changes(Stamp::new(2))?;

        vec.push(6);
        vec.stamped_write_with_changes(Stamp::new(3))?;

        vec.push(7);
        vec.stamped_write_with_changes(Stamp::new(4))?;

        vec.push(8);
        vec.stamped_write_with_changes(Stamp::new(5))?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);

        // Rollback before stamp 4 (should go to stamp 3)
        let _ = vec.rollback_before(Stamp::new(4))?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(vec.stamp(), Stamp::new(3));

        Ok(())
    }

    /// Regression test: rollback-after-rollback with delete_at losing entries.
    ///
    /// After the first rollback, restored entries sit in `updated.current`.
    /// If `delete_at` removes one from `updated.current` during reprocessing,
    /// and `serialize_changes` only iterated `updated.current` keys (the old bug),
    /// the entry's prev value would be lost from the change file.
    /// On a second rollback, the slot would contain stale on-disk data
    /// instead of the correct rolled-back value.
    fn run_rollback_after_rollback_with_delete<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        // Stamp 1 (baseline): [10, 20, 30, 40, 50]
        for &v in &[10, 20, 30, 40, 50] {
            vec.push(v);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.collect(), vec![10, 20, 30, 40, 50]);

        // Stamp 2: update slot 2 (30 → 99), delete slot 1 (creates hole)
        vec.update(2, 99)?;
        let _ = vec.take(1);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![10, 99, 40, 50]);

        // First rollback → back to stamp 1
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![10, 20, 30, 40, 50]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        // Now reprocess: delete slot 2 (the one we just restored), update slot 3
        // This simulates an address becoming empty during reprocessing
        let _ = vec.take(2); // removes 30 from updated.current
        vec.update(3, 88)?;
        vec.stamped_write_with_changes(Stamp::new(3))?;
        assert_eq!(vec.collect(), vec![10, 20, 88, 50]);

        // Second rollback → must go back to stamp 1 values
        vec.rollback()?;
        let result = vec.collect();
        assert_eq!(
            result,
            vec![10, 20, 30, 40, 50],
            "Second rollback must restore all original values. \
             Slot 2 was deleted during reprocessing but its prev value (30) \
             must still be tracked in the change file."
        );
        assert_eq!(vec.stamp(), Stamp::new(1));

        Ok(())
    }

    fn run_rollback_after_untracked_checkpoint<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        for erased in [false, true] {
            let (db, temp) = setup_db()?;
            let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;
            for i in 0..100 {
                vec.push(i);
            }
            vec.stamped_write_with_changes(Stamp::new(1))?;

            // An intermediate write may already have collected rollback values.
            vec.update(65, 650)?;
            vec.write()?;
            vec.update(65, 651)?;
            vec.update(66, 660)?;
            assert_eq!(vec.take(67), Some(67));
            vec.truncate_if_needed_at(90)?;
            vec.push(900);
            if erased {
                let stored: &mut dyn AnyStoredVec = &mut vec;
                stored.any_stamped_write_maybe_with_changes(Stamp::new(2), false)?;
            } else {
                vec.stamped_write_maybe_with_changes(Stamp::new(2), false)?;
            }
            vec.flush()?;
            assert_eq!(vec.find_rollback_files()?.len(), 1);
            let mut baseline: Vec<_> = (0..90).map(Some).chain([Some(900)]).collect();
            baseline[65] = Some(651);
            baseline[66] = Some(660);
            baseline[67] = None;
            assert_eq!(vec.collect_holed(), baseline);
            drop((vec, db));

            let db = Database::open(temp.path())?;
            let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;
            assert_eq!(vec.stamp(), Stamp::new(2));
            assert_eq!(vec.collect_holed(), baseline);
            vec.update(65, 999)?;
            vec.update(67, 670)?;
            vec.push(901);
            if erased {
                let stored: &mut dyn AnyStoredVec = &mut vec;
                stored.any_stamped_write_maybe_with_changes(Stamp::new(3), true)?;
            } else {
                vec.stamped_write_maybe_with_changes(Stamp::new(3), true)?;
            }
            vec.rollback()?;
            assert_eq!(vec.collect_holed(), baseline);
            assert_eq!(vec.stamp(), Stamp::new(2));
            vec.flush()?;
            drop((vec, db));

            let db = Database::open(temp.path())?;
            let (vec, _) = V::import_with_changes(&db, "test", 10)?;
            assert_eq!(vec.collect_holed(), baseline);
            assert_eq!(vec.stamp(), Stamp::new(2));
        }
        Ok(())
    }

    fn run_rollback_across_intermediate_writes<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        vec.push(10);
        vec.push(20);
        vec.stamped_write_with_changes(Stamp::new(1))?;

        vec.update(0, 11)?;
        vec.write()?;

        vec.push(30);
        vec.write()?;

        vec.update(0, 12)?;
        vec.update(2, 31)?;
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![12, 20, 31]);

        vec.rollback()?;
        assert_eq!(vec.collect(), vec![10, 20]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        Ok(())
    }

    fn run_holes_persist_only_when_changed<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, _) = V::import_with_changes(&db, "test", 10)?;

        vec.push(10);
        vec.push(20);
        vec.push(30);
        vec.stamped_write_with_changes(Stamp::new(1))?;

        let _ = vec.take(1);
        assert!(vec.is_dirty());
        assert!(vec.write()?);
        assert!(!vec.is_dirty());
        assert!(!vec.write()?);

        vec.update(1, 21)?;
        let _ = vec.take(2);
        assert!(vec.is_dirty());
        vec.reset_unsaved();

        assert_eq!(vec.collect_holed(), vec![Some(10), None, Some(30)]);
        assert!(!vec.is_dirty());
        assert!(!vec.write()?);

        Ok(())
    }

    fn run_rollback_persists_restored_holes<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        let (db, _temp) = setup_db()?;
        let (mut vec, options) = V::import_with_changes(&db, "test", 10)?;

        vec.push(10);
        vec.push(20);
        vec.push(30);
        let _ = vec.take(1);
        vec.stamped_write_with_changes(Stamp::new(1))?;

        vec.update(1, 21)?;
        let _ = vec.take(2);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect_holed(), vec![Some(10), Some(21), None]);

        vec.rollback()?;
        assert_eq!(vec.collect_holed(), vec![Some(10), None, Some(30)]);
        assert!(vec.is_dirty());
        vec.write()?;
        drop(vec);

        let vec = V::forced_import_with(options)?;
        assert_eq!(vec.collect_holed(), vec![Some(10), None, Some(30)]);

        Ok(())
    }

    // Test instantiation for each mutable raw vec type

    fn run<V: RollbackVec>() -> Result<()> {
        run_rollback_then_save_new_state::<V>()?;
        run_rollback_with_updates::<V>()?;
        run_rollback_with_holes::<V>()?;
        run_rollback_with_truncation_and_updates::<V>()?;
        run_rollback_with_holes_and_updates::<V>()?;
        run_multiple_updates_to_same_index::<V>()?;
        run_complex_mixed_operations::<V>()?;
        run_rollback_to_empty::<V>()?;
        run_reset::<V>()?;
        run_deep_rollback_chain::<V>()?;
        run_rollback_all_elements_updated::<V>()?;
        run_multiple_holes_then_rollback::<V>()?;
        run_rollback_before::<V>()?;
        run_rollback_after_rollback_with_delete::<V>()?;
        run_rollback_after_untracked_checkpoint::<V>()?;
        run_rollback_across_intermediate_writes::<V>()?;
        run_holes_persist_only_when_changed::<V>()?;
        run_rollback_persists_restored_holes::<V>()?;
        Ok(())
    }

    #[test]
    fn bytes() -> Result<()> {
        run::<MutableVec<BytesVec<usize, u32>>>()
    }

    #[cfg(feature = "zerocopy")]
    #[test]
    fn zerocopy() -> Result<()> {
        run::<MutableVec<ZeroCopyVec<usize, u32>>>()
    }
} // end mod raw_rollback

// PART 3: Checkpoint Rollback Tests (ALL vec types)

mod checkpoint_rollback {
    use vecdb::BytesVec;

    use super::*;

    #[cfg(any(feature = "zerocopy", feature = "pco"))]
    use vecdb::EagerVec;
    #[cfg(feature = "lz4")]
    use vecdb::LZ4Vec;
    #[cfg(feature = "pco")]
    use vecdb::PcoVec;
    #[cfg(feature = "zerocopy")]
    use vecdb::ZeroCopyVec;
    #[cfg(feature = "zstd")]
    use vecdb::ZstdVec;

    fn run<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let options = ImportOptions::new(&db, "test", Version::TWO).with_saved_stamped_changes(10);
        let mut vec = V::forced_import_with(options)?;

        for i in 0..100 {
            vec.push(i);
        }
        AnyStoredVec::any_stamped_write_maybe_with_changes(&mut vec, Stamp::new(1), false)?;

        vec.push(100);
        AnyStoredVec::any_stamped_write_maybe_with_changes(&mut vec, Stamp::new(2), true)?;
        WritableVec::rollback(&mut vec)?;

        assert_eq!(vec.collect(), (0..100).collect::<Vec<_>>());
        assert_eq!(AnyStoredVec::stamp(&vec), Stamp::new(1));

        Ok(())
    }

    #[test]
    fn bytes() -> Result<()> {
        run::<BytesVec<usize, u32>>()
    }

    #[cfg(feature = "zerocopy")]
    #[test]
    fn zerocopy() -> Result<()> {
        run::<ZeroCopyVec<usize, u32>>()
    }

    #[cfg(feature = "pco")]
    #[test]
    fn pco() -> Result<()> {
        run::<PcoVec<usize, u32>>()
    }

    #[cfg(feature = "lz4")]
    #[test]
    fn lz4() -> Result<()> {
        run::<LZ4Vec<usize, u32>>()
    }

    #[cfg(feature = "zstd")]
    #[test]
    fn zstd() -> Result<()> {
        run::<ZstdVec<usize, u32>>()
    }

    #[cfg(feature = "zerocopy")]
    #[test]
    fn eager_zerocopy() -> Result<()> {
        run::<EagerVec<ZeroCopyVec<usize, u32>>>()
    }

    #[cfg(feature = "pco")]
    #[test]
    fn eager_pco() -> Result<()> {
        run::<EagerVec<PcoVec<usize, u32>>>()
    }
}

// PART 4: Comprehensive Integration Test

// Complex rollback + flush + reopen test with file integrity verification.

mod integration {
    use vecdb::{BytesVec, MutableVec};

    use super::*;
    use crate::raw_rollback::RollbackVec;

    #[cfg(feature = "zerocopy")]
    use vecdb::ZeroCopyVec;

    /// Comprehensive integration test: rollback + flush + reopen with integrity verification.
    ///
    /// This test verifies that after rollback + flush + close + reopen:
    /// 1. Data can be correctly read back using individual gets
    /// 2. Data can be correctly read back using iterators
    /// 3. Redo operations produce the same readable state
    fn run_data_integrity_rollback_flush_reopen<V>() -> Result<()>
    where
        V: RollbackVec,
    {
        // Create database
        let (database, temp) = setup_db()?;

        let (mut vec, _) = V::import_with_changes(&database, "vec", 10)?;

        // Phase 1: Initial work
        for i in 0..5 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Phase 2: More work
        for i in 5..10 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(2))?;

        // Checkpoint 1
        let checkpoint1_data = vec.collect_holed();
        let checkpoint1_stamp = vec.stamp();

        // Phase 3: Three more operations with flush
        vec.update(2, 100)?;
        vec.update(7, 200)?;
        vec.stamped_write_with_changes(Stamp::new(3))?;

        vec.push(20);
        vec.push(21);
        vec.stamped_write_with_changes(Stamp::new(4))?;

        let _ = vec.take(5);
        vec.push(30);
        vec.stamped_write_with_changes(Stamp::new(5))?;

        // Checkpoint 2
        let checkpoint2_data = vec.collect_holed();
        let checkpoint2_stamp = vec.stamp();

        // Undo last 3 operations
        vec.rollback()?;
        vec.rollback()?;
        vec.rollback()?;

        // Verify in-memory data matches checkpoint1
        let after_undo_data = vec.collect_holed();
        let after_undo_stamp = vec.stamp();

        assert_eq!(after_undo_stamp, checkpoint1_stamp);
        assert_eq!(after_undo_data, checkpoint1_data);

        // Flush and close
        vec.stamped_write_with_changes(checkpoint1_stamp)?;

        database.flush()?;
        drop(vec);
        drop(database);
        let database = Database::open(temp.path())?;

        // Reopen
        let (mut vec, _) = V::import_with_changes(&database, "vec", 10)?;

        // Verify using individual gets
        let reader = vec.reader();
        let mut data_via_gets = Vec::new();
        for i in 0..vec.len() {
            let value = vec.get_with_reader(i, &reader);
            data_via_gets.push(value);
        }
        drop(reader);

        assert_eq!(data_via_gets, checkpoint1_data);

        // Verify using iterator
        let data_via_iter = vec.collect_holed();
        assert_eq!(data_via_iter, checkpoint1_data);

        // Redo the same 3 operations
        vec.update(2, 100)?;
        vec.update(7, 200)?;
        vec.stamped_write_with_changes(Stamp::new(3))?;

        vec.push(20);
        vec.push(21);
        vec.stamped_write_with_changes(Stamp::new(4))?;

        let _ = vec.take(5);
        vec.push(30);
        vec.stamped_write_with_changes(Stamp::new(5))?;

        // Verify in-memory data matches checkpoint2
        let after_redo_data = vec.collect_holed();
        let after_redo_stamp = vec.stamp();

        assert_eq!(after_redo_stamp, checkpoint2_stamp);
        assert_eq!(after_redo_data, checkpoint2_data);

        // Flush and close
        vec.stamped_write_with_changes(checkpoint2_stamp)?;
        database.flush()?;
        drop(vec);
        drop(database);
        let database = Database::open(temp.path())?;

        // Reopen again
        let (vec, _) = V::import_with_changes(&database, "vec", 10)?;

        // Verify using individual gets
        let reader = vec.reader();
        let mut data_via_gets = Vec::new();
        for i in 0..vec.len() {
            let value = vec.get_with_reader(i, &reader);
            data_via_gets.push(value);
        }
        drop(reader);

        assert_eq!(data_via_gets, checkpoint2_data);

        // Verify using iterator
        let data_via_iter = vec.collect_holed();
        assert_eq!(data_via_iter, checkpoint2_data);
        assert_eq!(
            vec.collect(),
            checkpoint2_data.into_iter().flatten().collect::<Vec<_>>()
        );

        Ok(())
    }

    #[cfg(feature = "zerocopy")]
    mod zerocopy {
        use super::*;

        type V = MutableVec<ZeroCopyVec<usize, u32>>;

        #[test]
        fn data_integrity_rollback_flush_reopen() -> Result<()> {
            run_data_integrity_rollback_flush_reopen::<V>()
        }
    }

    mod bytes {
        use super::*;

        type V = MutableVec<BytesVec<usize, u32>>;

        #[test]
        fn data_integrity_rollback_flush_reopen() -> Result<()> {
            run_data_integrity_rollback_flush_reopen::<V>()
        }
    }
}
