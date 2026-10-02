//! Mutable rollback, intermediate writes, branch replay, and reopen integrity.

use rawdb::Database;
use tempfile::TempDir;

use super::{MutableVec, raw::MutableRawVec};
use crate::{
    AnyStoredVec, AnyVec, ImportOptions, ReadableVec, Result, Stamp, StoredVec, Version,
    WritableVec,
};

fn setup_db() -> Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

fn import_with_changes<V: StoredVec<I = usize, T = u32>>(db: &Database, name: &str) -> Result<V> {
    V::forced_import_with(ImportOptions::new(db, name, Version::TWO).with_saved_stamped_changes(10))
}

mod mutation_rollback {
    use crate::BytesVec;
    #[cfg(feature = "zerocopy")]
    use crate::ZeroCopyVec;

    use super::*;

    fn run_multiple_updates_to_same_index<V>() -> Result<()>
    where
        V: MutableRawVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;

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
        V: MutableRawVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;

        // Stamp 1: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        for i in 0..10 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: Complex operations
        // - Delete indices 1, 3, 5
        // - Update indices 2, 6, 8
        // - Push new values 100, 101
        vec.delete(1);
        vec.delete(3);
        vec.delete(5);
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

    fn run_deep_rollback_chain<V>() -> Result<()>
    where
        V: MutableRawVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;

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

        vec.delete(1);
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
        assert_eq!(vec.stamp(), Stamp::new(1));
        vec.save_rollback_state();

        vec.extend(0..3);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        vec.update(0, 10)?;
        vec.stamped_write_with_changes(Stamp::new(3))?;
        vec.push(3);
        vec.stamped_write_with_changes(Stamp::new(4))?;
        assert_eq!(vec.rollback_before(Stamp::new(3))?, Stamp::new(2));
        assert_eq!(vec.collect(), [0, 1, 2]);
        vec.push(99);
        vec.stamped_write_with_changes(Stamp::new(3))?;
        assert_eq!(vec.collect(), [0, 1, 2, 99]);
        assert_eq!(vec.stamp(), Stamp::new(3));

        Ok(())
    }

    fn run_rollback_all_elements_updated<V>() -> Result<()>
    where
        V: MutableRawVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;

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
        V: MutableRawVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;

        // Stamp 1: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        for i in 0..10 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        // Stamp 2: Delete every other element
        for i in (0..10).step_by(2) {
            vec.delete(i);
        }
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![1, 3, 5, 7, 9]);

        // Rollback - should restore all deleted items
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

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
        V: MutableRawVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;

        // Stamp 1 (baseline): [10, 20, 30, 40, 50]
        for &v in &[10, 20, 30, 40, 50] {
            vec.push(v);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.collect(), vec![10, 20, 30, 40, 50]);

        // Stamp 2: update slot 2 (30 → 99), delete slot 1 (creates hole)
        vec.update(2, 99)?;
        vec.delete(1);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), vec![10, 99, 40, 50]);

        // First rollback → back to stamp 1
        vec.rollback()?;
        assert_eq!(vec.collect(), vec![10, 20, 30, 40, 50]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        // Now reprocess: delete slot 2 (the one we just restored), update slot 3
        // This simulates an address becoming empty during reprocessing
        vec.delete(2); // removes 30 from updated.current
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
        V: MutableRawVec<I = usize, T = u32>,
    {
        for erased in [false, true] {
            let (db, temp) = setup_db()?;
            let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;
            for i in 0..100 {
                vec.push(i);
            }
            vec.stamped_write_with_changes(Stamp::new(1))?;

            // An intermediate write may already have collected rollback values.
            vec.update(65, 650)?;
            vec.write()?;
            vec.update(65, 651)?;
            vec.update(66, 660)?;
            assert_eq!(vec.take(67, &vec.reader()), Some(67));
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
            let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;
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
            let vec = import_with_changes::<MutableVec<V>>(&db, "test")?;
            assert_eq!(vec.collect_holed(), baseline);
            assert_eq!(vec.stamp(), Stamp::new(2));
        }
        Ok(())
    }

    fn run_rollback_across_intermediate_writes<V>() -> Result<()>
    where
        V: MutableRawVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;

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

    fn run_holes_persistence_and_reset<V>() -> Result<()>
    where
        V: MutableRawVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;

        vec.push(10);
        vec.push(20);
        vec.push(30);
        vec.stamped_write_with_changes(Stamp::new(1))?;

        vec.delete(1);
        assert!(vec.is_dirty());
        assert!(vec.write()?);
        assert!(!vec.is_dirty());
        assert!(!vec.write()?);

        vec.update(1, 21)?;
        vec.delete(2);
        assert!(vec.is_dirty());
        vec.reset_unsaved();

        assert_eq!(vec.collect_holed(), vec![Some(10), None, Some(30)]);
        assert!(!vec.is_dirty());
        assert!(!vec.write()?);

        vec.update(0, 100)?;
        vec.delete(2);
        vec.push(40);
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.stored_len(), 3);
        assert_eq!(vec.pushed_len(), 1);
        vec.reset()?;
        assert!(vec.collect_holed().is_empty());
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.pushed_len(), 0);
        assert!(vec.holes().is_empty());
        assert!(vec.updated().is_empty());
        vec.extend([100, 101, 102]);
        vec.stamped_write_with_changes(Stamp::new(1))?;
        assert_eq!(vec.collect_holed(), [Some(100), Some(101), Some(102)]);
        assert_eq!(vec.stored_len(), 3);
        assert_eq!(vec.pushed_len(), 0);

        Ok(())
    }

    fn run_rollback_persists_restored_holes<V>() -> Result<()>
    where
        V: MutableRawVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<MutableVec<V>>(&db, "test")?;

        vec.push(10);
        vec.push(20);
        vec.push(30);
        vec.delete(1);
        vec.stamped_write_with_changes(Stamp::new(1))?;

        vec.update(1, 21)?;
        vec.delete(2);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect_holed(), vec![Some(10), Some(21), None]);

        vec.rollback()?;
        assert_eq!(vec.collect_holed(), vec![Some(10), None, Some(30)]);
        assert!(vec.is_dirty());
        vec.write()?;
        drop(vec);

        let vec = import_with_changes::<MutableVec<V>>(&db, "test")?;
        assert_eq!(vec.collect_holed(), vec![Some(10), None, Some(30)]);

        Ok(())
    }

    // Test instantiation for each mutable raw vec type

    fn run<V: MutableRawVec<I = usize, T = u32>>() -> Result<()> {
        run_multiple_updates_to_same_index::<V>()?;
        run_complex_mixed_operations::<V>()?;
        run_deep_rollback_chain::<V>()?;
        run_rollback_all_elements_updated::<V>()?;
        run_multiple_holes_then_rollback::<V>()?;
        run_rollback_after_rollback_with_delete::<V>()?;
        run_rollback_after_untracked_checkpoint::<V>()?;
        run_rollback_across_intermediate_writes::<V>()?;
        run_holes_persistence_and_reset::<V>()?;
        run_rollback_persists_restored_holes::<V>()?;
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
}

mod integration {
    use crate::BytesVec;

    use super::*;

    #[cfg(feature = "zerocopy")]
    use crate::ZeroCopyVec;

    fn assert_state<V: MutableRawVec<I = usize, T = u32>>(
        vec: &MutableVec<V>,
        expected: &[Option<u32>],
    ) {
        let reader = vec.reader();
        assert_eq!(
            (0..vec.len())
                .map(|i| vec.get_with_reader(i, &reader))
                .collect::<Vec<_>>(),
            expected
        );
        drop(reader);
        assert_eq!(vec.collect_holed(), expected);
        assert_eq!(
            vec.collect(),
            expected.iter().copied().flatten().collect::<Vec<_>>()
        );
    }

    /// Comprehensive integration test: rollback + flush + reopen with integrity verification.
    ///
    /// This test verifies that after rollback + flush + close + reopen:
    /// 1. Data can be correctly read back using individual gets
    /// 2. Data can be correctly read back using iterators
    /// 3. Redo operations produce the same readable state
    fn run_data_integrity_rollback_flush_reopen<V>() -> Result<()>
    where
        V: MutableRawVec<I = usize, T = u32>,
    {
        // Create database
        let (database, temp) = setup_db()?;

        let mut vec = import_with_changes::<MutableVec<V>>(&database, "vec")?;

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
        assert_eq!(checkpoint1_data, (0..10).map(Some).collect::<Vec<_>>());
        let checkpoint1_stamp = vec.stamp();

        // Phase 3: Three more operations with flush
        vec.update(2, 100)?;
        vec.update(7, 200)?;
        vec.stamped_write_with_changes(Stamp::new(3))?;

        vec.push(20);
        vec.push(21);
        vec.stamped_write_with_changes(Stamp::new(4))?;

        vec.delete(5);
        vec.push(30);
        vec.stamped_write_with_changes(Stamp::new(5))?;

        // Checkpoint 2
        let checkpoint2_data = vec.collect_holed();
        let mut expected = [0, 1, 100, 3, 4, 5, 6, 200, 8, 9, 20, 21, 30].map(Some);
        expected[5] = None;
        assert_eq!(checkpoint2_data, expected);
        let checkpoint2_stamp = vec.stamp();

        // Undo last 3 operations and finish the rewind before replaying.
        assert_eq!(vec.rollback_before(Stamp::new(3))?, checkpoint1_stamp);

        // Verify in-memory data matches checkpoint1
        let after_undo_data = vec.collect_holed();
        let after_undo_stamp = vec.stamp();

        assert_eq!(after_undo_stamp, checkpoint1_stamp);
        assert_eq!(after_undo_data, checkpoint1_data);

        // Flush and close
        vec.stamped_write(checkpoint1_stamp)?;

        database.flush()?;
        drop(vec);
        drop(database);
        let database = Database::open(temp.path())?;

        // Reopen
        let mut vec = import_with_changes::<MutableVec<V>>(&database, "vec")?;

        assert_state(&vec, &checkpoint1_data);

        // Redo the same 3 operations
        vec.update(2, 100)?;
        vec.update(7, 200)?;
        vec.stamped_write_with_changes(Stamp::new(3))?;

        vec.push(20);
        vec.push(21);
        vec.stamped_write_with_changes(Stamp::new(4))?;

        vec.delete(5);
        vec.push(30);
        vec.stamped_write_with_changes(Stamp::new(5))?;

        // Verify in-memory data matches checkpoint2
        let after_redo_data = vec.collect_holed();
        let after_redo_stamp = vec.stamp();

        assert_eq!(after_redo_stamp, checkpoint2_stamp);
        assert_eq!(after_redo_data, checkpoint2_data);

        // Flush and close
        vec.stamped_write(checkpoint2_stamp)?;
        database.flush()?;
        drop(vec);
        drop(database);
        let database = Database::open(temp.path())?;

        // Reopen again
        let mut vec = import_with_changes::<MutableVec<V>>(&database, "vec")?;

        assert_state(&vec, &checkpoint2_data);
        assert_eq!(vec.rollback_before(Stamp::new(2))?, Stamp::new(1));
        assert_state(&vec, &(0..5).map(Some).collect::<Vec<_>>());

        Ok(())
    }

    #[cfg(feature = "zerocopy")]
    mod zerocopy {
        use super::*;

        type V = ZeroCopyVec<usize, u32>;

        #[test]
        fn data_integrity_rollback_flush_reopen() -> Result<()> {
            run_data_integrity_rollback_flush_reopen::<V>()
        }
    }

    mod bytes {
        use super::*;

        type V = BytesVec<usize, u32>;

        #[test]
        fn data_integrity_rollback_flush_reopen() -> Result<()> {
            run_data_integrity_rollback_flush_reopen::<V>()
        }
    }
}
