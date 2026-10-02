//! Append-only rollback, branch replay, and checkpoint baselines across storage backends.

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{AnyStoredVec, ImportOptions, Result, Stamp, StoredVec, Version, WritableVec};

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

    fn import_with_changes<V>(db: &Database, name: &str) -> Result<V>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let mut options: ImportOptions = (db, name, Version::TWO).into();
        options = options.with_saved_stamped_changes(10);
        V::forced_import_with(options)
    }

    fn run_rollback_replay_and_reopen<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<V>(&db, "test")?;

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
        assert_eq!(vec.rollback_before(Stamp::new(2))?, Stamp::new(1));
        assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        vec.push(99);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect(), [0, 1, 2, 3, 4, 99]);
        assert_eq!(vec.stamp(), Stamp::new(2));
        assert_eq!(vec.rollback_before(Stamp::new(2))?, Stamp::new(1));
        vec.stamped_write(Stamp::new(1))?;
        drop(vec);
        let vec = import_with_changes::<V>(&db, "test")?;
        assert_eq!(vec.collect(), [0, 1, 2, 3, 4]);
        assert_eq!(vec.stamp(), Stamp::new(1));

        Ok(())
    }

    fn run_rollback_before<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<V>(&db, "test")?;

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
        let mut vec = import_with_changes::<V>(&db, "test")?;

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
        assert_eq!(vec.stamp(), Stamp::new(1));

        Ok(())
    }

    fn run_reset<V>() -> Result<()>
    where
        V: StoredVec<I = usize, T = u32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<V>(&db, "test")?;

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
        run_rollback_replay_and_reopen::<V>()?;
        run_rollback_before::<V>()?;
        run_deep_rollback_chain::<V>()?;
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

// PART 2: Checkpoint Rollback Tests (ALL vec types)

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
