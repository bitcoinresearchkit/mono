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

    use super::*;

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

    // Test instantiation for each mutable raw vec type

    fn run<V: MutableRawVec<I = usize, T = u32>>() -> Result<()> {
        run_rollback_after_rollback_with_delete::<V>()?;
        run_rollback_after_untracked_checkpoint::<V>()?;
        run_rollback_across_intermediate_writes::<V>()?;

        Ok(())
    }

    #[test]
    fn bytes() -> Result<()> {
        run::<BytesVec<usize, u32>>()
    }
}

mod integration {
    use crate::BytesVec;

    use super::*;

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

    mod bytes {
        use super::*;

        type V = BytesVec<usize, u32>;

        #[test]
        fn data_integrity_rollback_flush_reopen() -> Result<()> {
            run_data_integrity_rollback_flush_reopen::<V>()
        }
    }
}
