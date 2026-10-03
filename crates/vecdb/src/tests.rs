//! Critical invariants: rollback and recovery across vec kinds — mutable rollbacks, truncation and reopen,
//! computed vecs after a source shrinks or changes version, bulk mutable updates, overflow sidecars, compressed
//! pages, regions retained at startup, and reader caches after a branch is replaced.

use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use crate::{Budgeted, CacheBudget};

static TEST_LOCK: Mutex<()> = Mutex::new(());

pub fn init_cache() -> &'static CacheBudget {
    static CACHE: OnceLock<&'static CacheBudget> = OnceLock::new();
    CACHE.get_or_init(|| Budgeted::init_global(512 * 1024).unwrap())
}

/// Tests share the process-wide cache budget and some assert on it, so they run one at a time.
pub fn serial() -> MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

mod mutable {
    //! Mutable rollback, intermediate writes, branch replay, and reopen integrity.

    use rawdb::Database;
    use tempfile::TempDir;

    use crate::variants::mutable::{MutableVec, raw::MutableRawVec};
    use crate::{
        AnyStoredVec, AnyVec, ImportOptions, ReadableVec, Result, Stamp, StoredVec, Version,
        WritableVec,
    };

    fn setup_db() -> Result<(Database, TempDir)> {
        let temp = TempDir::new()?;
        let db = Database::open(temp.path())?;
        Ok((db, temp))
    }

    fn import_with_changes<V: StoredVec<I = usize, T = u32>>(
        db: &Database,
        name: &str,
    ) -> Result<V> {
        V::forced_import_with(
            ImportOptions::new(db, name, Version::TWO).with_saved_stamped_changes(10),
        )
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
            let _serial = crate::tests::serial();
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
                let _serial = crate::tests::serial();
                run_data_integrity_rollback_flush_reopen::<V>()
            }
        }
    }
}

mod truncation {
    //! Truncation rollback across native writes and reopening.

    use crate::{ImportOptions, Result, Stamp, StoredVec, Version};
    use rawdb::Database;
    use tempfile::TempDir;

    fn setup_db() -> Result<(Database, TempDir)> {
        let temp = TempDir::new()?;
        let db = Database::open(temp.path())?;
        Ok((db, temp))
    }

    fn import_with_changes<V>(db: &Database, name: &str) -> Result<V>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let mut options: ImportOptions = (db, name, Version::TWO).into();
        options = options.with_saved_stamped_changes(10);
        V::forced_import_with(options)
    }

    fn run_truncate_rollback_and_reopen<V: StoredVec<I = usize, T = u32>>() -> Result<()> {
        for (count, truncate_to, replacement) in [(8, 5, &[][..]), (5, 3, &[100, 200][..])] {
            let (db, _temp) = setup_db()?;
            let mut vec = import_with_changes::<V>(&db, "values")?;
            for value in 0..count {
                vec.push(value);
            }
            vec.stamped_write_with_changes(Stamp::new(1))?;
            vec.truncate_if_needed_at(truncate_to as usize)?;
            for &value in replacement {
                vec.push(value);
            }
            vec.stamped_write_with_changes(Stamp::new(2))?;
            assert_eq!(
                vec.collect(),
                (0..truncate_to)
                    .chain(replacement.iter().copied())
                    .collect::<Vec<_>>()
            );
            assert_eq!(vec.rollback_before(Stamp::new(2))?, Stamp::new(1));
            assert_eq!(vec.len(), count as usize);
            assert_eq!(vec.collect(), (0..count).collect::<Vec<_>>());
            assert_eq!(vec.stamp(), Stamp::new(1));

            for stamp in [Stamp::new(1), Stamp::new(2)] {
                vec.stamped_write(stamp)?;
                assert_eq!(vec.stored_len(), count as usize);
                assert_eq!(vec.collect(), (0..count).collect::<Vec<_>>());
                drop(vec);
                vec = import_with_changes::<V>(&db, "values")?;
                assert_eq!(vec.stamp(), stamp);
                assert_eq!(vec.stored_len(), count as usize);
                assert_eq!(vec.collect(), (0..count).collect::<Vec<_>>());
            }
        }
        Ok(())
    }

    macro_rules! instantiate_for {
        ($mod:ident, $ty:ty) => {
            mod $mod {
                use super::*;
                type V = $ty;

                #[test]
                fn truncate_rollback_reflush_and_reopen() -> Result<()> {
                    let _serial = crate::tests::serial();
                    run_truncate_rollback_and_reopen::<V>()
                }
            }
        };
    }

    instantiate_for!(bytes, crate::BytesVec<usize, u32>);

    instantiate_for!(
        mutable_bytes,
        crate::MutableVec<crate::BytesVec<usize, u32>>
    );

    #[cfg(feature = "pco")]
    instantiate_for!(pco, crate::PcoVec<usize, u32>);
}

mod recovery {
    #[cfg(feature = "pco")]
    use crate::PcoVec;
    use crate::tests::init_cache;
    use crate::{
        AnyStoredVec, AnyVec, Budgeted, BytesVec, Database, EagerVec, ImportableVec, ReadableVec,
        StoredVec, Version, WritableVec,
    };
    use brk_exit::Exit;
    use tempfile::tempdir;

    #[derive(Clone, Copy, Debug)]
    enum Compute {
        Batched,
        Transform,
    }

    fn check_recovery<V: StoredVec<I = usize, T = u64>>(compute: Compute) {
        init_cache();
        let exit = Exit::new();
        for max_from in [3, 99] {
            let directory = tempdir().unwrap();
            let mut previous_values: &[u64] = &[];
            for (expected, version) in [
                (&[0, 1, 2, 3, 4][..], Version::ONE),
                (&[0, 1, 2][..], Version::ONE),
                (&[0, 1, 2, 103, 104][..], Version::ONE),
                (&[][..], Version::ONE),
                (&[][..], Version::TWO),
                (&[200, 201, 202, 203, 204][..], Version::TWO),
            ] {
                let to = expected.len();
                let dependency_version;
                {
                    let db = Database::open(directory.path()).unwrap();
                    let mut source =
                        EagerVec::<BytesVec<usize, u64>>::forced_import(&db, "source", version)
                            .unwrap();
                    source.truncate_if_needed_at(0).unwrap();
                    for &value in expected {
                        source.push(value);
                    }
                    source.write().unwrap();
                    dependency_version = source.version();
                    let mut output =
                        EagerVec::<V>::forced_import(&db, "output", Version::ONE).unwrap();
                    let previous = output.collect();
                    let start =
                        if output.version() == output.header().vec_version() + dependency_version {
                            output.len().min(max_from).min(to)
                        } else {
                            0
                        };
                    let mut appended = 0;
                    match compute {
                        Compute::Batched => output.compute_batched_to(
                            max_from,
                            to,
                            dependency_version,
                            2,
                            |output, range| {
                                for index in range {
                                    output.push(expected[index]);
                                    appended += 1;
                                }
                                Ok(())
                            },
                            &exit,
                        ),
                        Compute::Transform => output.compute_transform(
                            max_from,
                            &source,
                            |(index, value, _)| {
                                appended += 1;
                                (index, value)
                            },
                            &exit,
                        ),
                    }
                    .unwrap();
                    assert_eq!(appended, to - start, "{compute:?}: appended rows");
                    assert_eq!(output.len(), to);
                    assert_eq!(output.collect().as_slice(), expected);
                    assert_eq!(previous.as_slice(), previous_values);
                    // No extra write/flush: the compute call must publish truncation itself.
                }
                let db = Database::open(directory.path()).unwrap();
                let output = EagerVec::<V>::forced_import(&db, "output", Version::ONE).unwrap();
                assert_eq!(output.len(), to, "reopened length at max_from={max_from}");
                assert_eq!(output.collect_range_at(0, to), expected);
                assert_eq!(
                    output.version(),
                    output.header().vec_version() + dependency_version
                );
                previous_values = expected;
            }
        }
    }

    #[test]
    fn raw_batched_recovery() {
        let _serial = crate::tests::serial();
        check_recovery::<BytesVec<usize, u64>>(Compute::Batched);
        check_recovery::<BytesVec<usize, u64, Budgeted>>(Compute::Batched);
    }

    #[cfg(feature = "pco")]
    #[test]
    fn pco_batched_recovery() {
        let _serial = crate::tests::serial();
        check_recovery::<PcoVec<usize, u64>>(Compute::Batched);
        check_recovery::<PcoVec<usize, u64, Budgeted>>(Compute::Batched);
    }

    #[test]
    fn zero_append_compute_transform_recovery() {
        let _serial = crate::tests::serial();
        check_recovery::<BytesVec<usize, u64>>(Compute::Transform);
        #[cfg(feature = "pco")]
        check_recovery::<PcoVec<usize, u64>>(Compute::Transform);
    }
}

mod update_many {
    use crate::{
        AnyStoredVec, BytesVec, Database, ImportOptions, ImportableVec, MutableVec, Result, Stamp,
        Version, WritableVec,
    };
    use tempfile::tempdir;

    #[test]
    fn update_many_preserves_stamped_rollback() -> Result<()> {
        let _serial = crate::tests::serial();
        for stored_len in [0, 8, 16] {
            let directory = tempdir()?;
            let database = Database::open(directory.path())?;
            let options =
                ImportOptions::new(&database, "values", Version::ONE).with_saved_stamped_changes(4);
            let mut vec = MutableVec::<BytesVec<usize, u32>>::import_with(options)?;
            vec.fill_to(stored_len, 0)?;
            vec.stamped_write_with_changes(Stamp::new(1))?;
            vec.fill_to(16, 0)?;
            vec.update(1, 40)?;

            // Sorted replacements span both sides of the stored/appended boundary,
            // overwrite a pending mutation, and preserve last-wins duplicates.
            vec.update_many([(0, 100), (1, 101), (1, 102), (7, 103), (8, 104), (15, 105)])?;
            let mut expected = vec![Some(0); 16];
            for (index, value) in [(0, 100), (1, 102), (7, 103), (8, 104), (15, 105)] {
                expected[index] = Some(value);
            }
            assert_eq!(vec.collect_holed(), expected);
            assert!(vec.update_many([(0, 2), (16, 3)]).is_err());
            assert_eq!(vec.collect_holed(), expected);
            vec.stamped_write_with_changes(Stamp::new(2))?;
            assert_eq!(vec.collect_holed(), expected);

            vec.rollback()?;
            assert_eq!(vec.stamp(), Stamp::new(1));
            assert_eq!(vec.collect_holed(), vec![Some(0); stored_len]);
        }
        Ok(())
    }
}

mod overflow {
    use std::fs;

    use crate::{
        AnyStoredVec, Bytes, Database, Error, ImportOptions, ImportableVec, OverflowVec,
        OverflowVecValue, ReadableVec, Result, Stamp, Version, WritableVec,
    };
    use tempfile::tempdir;

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

    #[test]
    fn rollback_and_truncation_keep_sidecar_in_sync() -> Result<()> {
        let _serial = crate::tests::serial();
        let temp = tempdir()?;
        let db = Database::open(temp.path())?;
        let options =
            ImportOptions::new(&db, "rollback", Version::ONE).with_saved_stamped_changes(5);
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
    fn sidecar_undo_is_prepared_before_either_half_is_overwritten() -> Result<()> {
        let _serial = crate::tests::serial();
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
}

mod grouped_pco {
    #![cfg(feature = "pco")]

    use crate::{
        AnyStoredVec, AnyVec, Database, ImportableVec, PcoVec, ReadableVec, Result, Version,
        WritableVec,
    };
    use tempfile::tempdir;

    const VALUES_PER_PAGE: usize = 8 * 1024 / size_of::<u64>();

    #[test]
    fn shared_chunks_stop_at_metadata_blocks_and_rebuild_from_chunk_boundaries() -> Result<()> {
        let _serial = crate::tests::serial();
        let temp = tempdir()?;
        let db = Database::open(temp.path())?;
        let mut vec = PcoVec::<usize, u64>::forced_import(&db, "values", Version::ONE)?;
        let initial: Vec<_> = (0..VALUES_PER_PAGE * 20 + 137)
            .map(|index| (index as u64).wrapping_mul(6364136223846793005))
            .collect();
        for &value in &initial {
            vec.push(value);
        }
        vec.write()?;

        // 21 pages occupy one full 136-byte metadata block and five records in
        // the second block: 136 + 8-byte base + 5 * 8-byte records.
        let pages = db.get_region(&vec.region_names()[1]).expect("pages region");
        assert_eq!(pages.meta().byte_len(), 184);
        assert_eq!(
            vec.collect_range_at(VALUES_PER_PAGE * 15 - 7, VALUES_PER_PAGE * 16 + 7),
            initial[VALUES_PER_PAGE * 15 - 7..VALUES_PER_PAGE * 16 + 7]
        );

        let truncate_at = VALUES_PER_PAGE * 9 + 31;
        vec.truncate_if_needed_at(truncate_at)?;
        let appended: Vec<_> = (0..VALUES_PER_PAGE * 3 + 71)
            .map(|index| (index as u64).wrapping_mul(1442695040888963407))
            .collect();
        for &value in &appended {
            vec.push(value);
        }
        vec.write()?;

        let expected = initial[..truncate_at]
            .iter()
            .chain(&appended)
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(vec.collect(), expected);
        drop(vec);

        let vec = PcoVec::<usize, u64>::import(&db, "values", Version::ONE)?;
        assert_eq!(vec.collect(), expected);
        Ok(())
    }
}

mod retain_regions {
    #![cfg(feature = "pco")]

    use crate::{
        AnyStoredVec, AnyVec, BytesVec, Database, ImportableVec, MutableVec, PcoVec, ReadableVec,
        Result, Version, WritableVec,
    };
    use tempfile::tempdir;

    #[test]
    fn retains_compressed_pages_and_mutable_holes() -> Result<()> {
        let _serial = crate::tests::serial();
        let directory = tempdir()?;
        {
            let database = Database::open(directory.path())?;
            let _ = database.create_region_if_needed("stale")?;

            let mut compressed =
                PcoVec::<usize, u64>::forced_import(&database, "compressed", Version::ONE)?;
            compressed.push(10);
            compressed.push(20);
            compressed.write()?;

            let mut mutable = MutableVec::<BytesVec<usize, u64>>::forced_import(
                &database,
                "mutable",
                Version::ONE,
            )?;
            mutable.push(1);
            mutable.push(2);
            mutable.push(3);
            mutable.write()?;
            mutable.delete(1);
            mutable.write()?;
        }

        let database = Database::open(directory.path())?;
        let compressed = PcoVec::<usize, u64>::import(&database, "compressed", Version::ONE)?;
        let mutable =
            MutableVec::<BytesVec<usize, u64>>::import(&database, "mutable", Version::ONE)?;
        let expected_regions = compressed
            .region_names()
            .into_iter()
            .chain(mutable.region_names())
            .collect::<Vec<_>>();

        database.retain_accessed_regions()?;

        assert!(database.get_region("stale").is_none());
        assert!(
            expected_regions
                .iter()
                .all(|name| database.get_region(name).is_some())
        );
        assert_eq!(compressed.collect_range_at(0, 2), [10, 20]);
        assert_eq!(mutable.collect_holed(), [Some(1), None, Some(3)]);

        Ok(())
    }
}

mod source_ranges {
    #[cfg(feature = "pco")]
    use crate::PcoVec;
    use crate::tests::init_cache;

    use crate::{AnyVec, Budgeted, BytesVec, Database, ReadableVec, StoredVec, Version};
    use tempfile::tempdir;

    fn check_source<V: StoredVec<I = usize, T = u64>>() {
        const LEN: usize = 12_345;
        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let budget = init_cache();
        let mut source = V::import(&db, "source", Version::ONE).unwrap();
        let captured = source.read_only_clone();
        assert!(captured.is_empty());
        for i in 0..LEN {
            source.push(i as u64);
        }
        assert!(captured.is_empty());
        source.write().unwrap();
        assert_eq!(captured.collect_one_at(42), Some(42));
        assert!(!source.read_cached_into_at(0, LEN, &mut Vec::new()));
        let indices = [42, 42, 1023, 1024, 8191, 8192, LEN - 1, LEN];
        assert_eq!(
            captured.read_sorted_at(&indices),
            indices[..7].iter().map(|&i| i as u64).collect::<Vec<_>>()
        );
        let expected: Vec<_> = (0..LEN as u64).collect();
        assert_eq!(source.collect(), expected);
        assert!(captured.read_cached_into_at(0, LEN, &mut Vec::new()));
        budget.clear();
        assert!(!captured.read_cached_into_at(0, LEN, &mut Vec::new()));
        assert_eq!(captured.collect(), expected);

        source.push(LEN as u64);
        assert_eq!(captured.len(), LEN);
        assert_eq!(captured.collect_one_at(LEN), None);
        source.write().unwrap();
        assert!(captured.read_cached_into_at(0, LEN, &mut Vec::new()));
        assert!(captured.read_cached_into_at(LEN, LEN + 1, &mut Vec::new()));
        assert_eq!(captured.collect_one_at(LEN), Some(LEN as u64));

        for from in [8193, 1023, 0] {
            source.truncate_if_needed_at(from).unwrap();
            if from != 0 {
                assert!(captured.read_cached_into_at(0, from, &mut Vec::new()));
            }
            for i in from..=LEN {
                source.push((i + from + 1) as u64);
            }
            source.write().unwrap();
            let expected: Vec<_> = (0..=LEN)
                .map(|i| {
                    if i < from {
                        i as u64
                    } else {
                        (i + from + 1) as u64
                    }
                })
                .collect();
            assert_eq!(captured.collect(), expected);
            assert_eq!(
                source.fold_range_at(0, LEN + 1, 0u64, |sum, value| sum + value),
                expected.iter().sum::<u64>()
            );
        }
        source.reset().unwrap();
        assert!(captured.collect().is_empty());
        source.push(99);
        source.write().unwrap();
        assert_eq!(captured.collect(), [99]);
        drop(captured);
        drop(source);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn raw_source_ranges() {
        let _serial = crate::tests::serial();
        check_source::<BytesVec<usize, u64, Budgeted>>();
    }

    #[cfg(feature = "pco")]
    #[test]
    fn pco_source_ranges() {
        let _serial = crate::tests::serial();
        check_source::<PcoVec<usize, u64, Budgeted>>();
    }
}
