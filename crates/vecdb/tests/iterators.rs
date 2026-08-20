//! Generic iterator tests for all vec types.
//!
//! These tests run against any type implementing `StoredVec`, ensuring
//! consistent iterator behavior across BytesVec, ZeroCopyVec, PcoVec, LZ4Vec, and ZstdVec.

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{StoredVec, Version};

// ============================================================================
// Test Setup Helpers
// ============================================================================

fn setup_db() -> vecdb::Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

// ============================================================================
// Generic Clean Iterator Tests
// ============================================================================

mod clean_iter {
    use super::*;

    fn run_basic<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        let collected: Vec<i32> = vec.collect();
        assert_eq!(collected.len(), 100);
        assert_eq!(collected[0], 0);
        assert_eq!(collected[99], 99);
        Ok(())
    }

    fn run_nth<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        // Test point reads at specific indices
        assert_eq!(vec.collect_first().unwrap(), 0);
        assert_eq!(vec.collect_one(10).unwrap(), 10);
        assert_eq!(vec.collect_one(11).unwrap(), 11);
        Ok(())
    }

    fn run_skip<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        let collected = vec.collect_range(50, 100);
        assert_eq!(collected.len(), 50);
        assert_eq!(collected[0], 50);
        Ok(())
    }

    fn run_take<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        let collected = vec.collect_range(0, 25);
        assert_eq!(collected.len(), 25);
        assert_eq!(collected[24], 24);
        Ok(())
    }

    fn run_set_position<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        let collected = vec.collect_range(50, 52);
        assert_eq!(collected[0], 50);
        assert_eq!(collected[1], 51);
        Ok(())
    }

    fn run_set_end<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        let collected = vec.collect_range(0, 50);
        assert_eq!(collected.len(), 50);
        Ok(())
    }

    fn run_last<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        assert_eq!(vec.collect_last().unwrap(), 99);
        Ok(())
    }

    fn run_last_empty<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let vec = V::forced_import(&db, "test", Version::ONE)?;

        assert_eq!(vec.len(), 0);
        assert!(vec.collect().is_empty());
        Ok(())
    }

    fn run_exact_size<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        assert_eq!(vec.len(), 100);
        assert_eq!(vec.collect_range(0, 100).len(), 100);
        assert_eq!(vec.collect_range(1, 100).len(), 99);
        Ok(())
    }

    fn run_buffer_crossing<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..10000 {
            vec.push(i);
        }
        vec.write()?;

        let collected: Vec<i32> = vec.collect();
        assert_eq!(collected.len(), 10000);

        for (i, &val) in collected.iter().enumerate() {
            assert_eq!(val, i as i32);
        }
        Ok(())
    }

    fn run_multiple_skip_take<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..1000 {
            vec.push(i);
        }
        vec.write()?;

        // skip(100).take(200).skip(50).take(100) => collect_range(150, 250)
        let collected = vec.collect_range(150, 250);

        assert_eq!(collected.len(), 100);
        assert_eq!(collected[0], 150);
        assert_eq!(collected[99], 249);
        Ok(())
    }

    fn run_set_position_multiple<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..1000 {
            vec.push(i);
        }
        vec.write()?;

        // Random access reads at various positions
        assert_eq!(vec.collect_one(100).unwrap(), 100);
        assert_eq!(vec.collect_one(500).unwrap(), 500);
        assert_eq!(vec.collect_one(50).unwrap(), 50);
        Ok(())
    }

    fn run_nth_beyond_end<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..10 {
            vec.push(i);
        }
        vec.write()?;

        // Reading beyond end returns empty
        let collected = vec.collect_range(10, 10);
        assert!(collected.is_empty());
        Ok(())
    }

    fn run_skip_all<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        let collected = vec.collect_range(100, 100);
        assert!(collected.is_empty());
        Ok(())
    }

    fn run_take_zero<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        let collected = vec.collect_range(0, 0);
        assert_eq!(collected.len(), 0);
        Ok(())
    }

    fn run_size_hint_consistency<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        // Test that collect_range returns correct sizes for progressive ranges
        for i in 0..100 {
            let remaining = vec.collect_range(i, 100);
            assert_eq!(remaining.len(), 100 - i);
        }
        Ok(())
    }

    // ============================================================================
    // Test instantiation for each vec type
    // ============================================================================

    mod bytes {
        use super::*;
        use vecdb::BytesVec;
        type V = BytesVec<usize, i32>;

        #[test]
        fn basic() -> vecdb::Result<()> {
            run_basic::<V>()
        }
        #[test]
        fn nth() -> vecdb::Result<()> {
            run_nth::<V>()
        }
        #[test]
        fn skip() -> vecdb::Result<()> {
            run_skip::<V>()
        }
        #[test]
        fn take() -> vecdb::Result<()> {
            run_take::<V>()
        }
        #[test]
        fn set_position() -> vecdb::Result<()> {
            run_set_position::<V>()
        }
        #[test]
        fn set_end() -> vecdb::Result<()> {
            run_set_end::<V>()
        }
        #[test]
        fn last() -> vecdb::Result<()> {
            run_last::<V>()
        }
        #[test]
        fn last_empty() -> vecdb::Result<()> {
            run_last_empty::<V>()
        }
        #[test]
        fn exact_size() -> vecdb::Result<()> {
            run_exact_size::<V>()
        }
        #[test]
        fn buffer_crossing() -> vecdb::Result<()> {
            run_buffer_crossing::<V>()
        }
        #[test]
        fn multiple_skip_take() -> vecdb::Result<()> {
            run_multiple_skip_take::<V>()
        }
        #[test]
        fn set_position_multiple() -> vecdb::Result<()> {
            run_set_position_multiple::<V>()
        }
        #[test]
        fn nth_beyond_end() -> vecdb::Result<()> {
            run_nth_beyond_end::<V>()
        }
        #[test]
        fn skip_all() -> vecdb::Result<()> {
            run_skip_all::<V>()
        }
        #[test]
        fn take_zero() -> vecdb::Result<()> {
            run_take_zero::<V>()
        }
        #[test]
        fn size_hint_consistency() -> vecdb::Result<()> {
            run_size_hint_consistency::<V>()
        }
    }

    #[cfg(feature = "zerocopy")]
    mod zerocopy {
        use super::*;
        use vecdb::ZeroCopyVec;
        type V = ZeroCopyVec<usize, i32>;

        #[test]
        fn basic() -> vecdb::Result<()> {
            run_basic::<V>()
        }
        #[test]
        fn nth() -> vecdb::Result<()> {
            run_nth::<V>()
        }
        #[test]
        fn skip() -> vecdb::Result<()> {
            run_skip::<V>()
        }
        #[test]
        fn take() -> vecdb::Result<()> {
            run_take::<V>()
        }
        #[test]
        fn set_position() -> vecdb::Result<()> {
            run_set_position::<V>()
        }
        #[test]
        fn set_end() -> vecdb::Result<()> {
            run_set_end::<V>()
        }
        #[test]
        fn last() -> vecdb::Result<()> {
            run_last::<V>()
        }
        #[test]
        fn last_empty() -> vecdb::Result<()> {
            run_last_empty::<V>()
        }
        #[test]
        fn exact_size() -> vecdb::Result<()> {
            run_exact_size::<V>()
        }
        #[test]
        fn buffer_crossing() -> vecdb::Result<()> {
            run_buffer_crossing::<V>()
        }
        #[test]
        fn multiple_skip_take() -> vecdb::Result<()> {
            run_multiple_skip_take::<V>()
        }
        #[test]
        fn set_position_multiple() -> vecdb::Result<()> {
            run_set_position_multiple::<V>()
        }
        #[test]
        fn nth_beyond_end() -> vecdb::Result<()> {
            run_nth_beyond_end::<V>()
        }
        #[test]
        fn skip_all() -> vecdb::Result<()> {
            run_skip_all::<V>()
        }
        #[test]
        fn take_zero() -> vecdb::Result<()> {
            run_take_zero::<V>()
        }
        #[test]
        fn size_hint_consistency() -> vecdb::Result<()> {
            run_size_hint_consistency::<V>()
        }
    }

    #[cfg(feature = "pco")]
    mod pco {
        use super::*;
        use vecdb::PcoVec;
        type V = PcoVec<usize, i32>;

        #[test]
        fn basic() -> vecdb::Result<()> {
            run_basic::<V>()
        }
        #[test]
        fn nth() -> vecdb::Result<()> {
            run_nth::<V>()
        }
        #[test]
        fn skip() -> vecdb::Result<()> {
            run_skip::<V>()
        }
        #[test]
        fn take() -> vecdb::Result<()> {
            run_take::<V>()
        }
        #[test]
        fn set_position() -> vecdb::Result<()> {
            run_set_position::<V>()
        }
        #[test]
        fn set_end() -> vecdb::Result<()> {
            run_set_end::<V>()
        }
        #[test]
        fn last() -> vecdb::Result<()> {
            run_last::<V>()
        }
        #[test]
        fn last_empty() -> vecdb::Result<()> {
            run_last_empty::<V>()
        }
        #[test]
        fn exact_size() -> vecdb::Result<()> {
            run_exact_size::<V>()
        }
        #[test]
        fn buffer_crossing() -> vecdb::Result<()> {
            run_buffer_crossing::<V>()
        }
        #[test]
        fn multiple_skip_take() -> vecdb::Result<()> {
            run_multiple_skip_take::<V>()
        }
        #[test]
        fn set_position_multiple() -> vecdb::Result<()> {
            run_set_position_multiple::<V>()
        }
        #[test]
        fn nth_beyond_end() -> vecdb::Result<()> {
            run_nth_beyond_end::<V>()
        }
        #[test]
        fn skip_all() -> vecdb::Result<()> {
            run_skip_all::<V>()
        }
        #[test]
        fn take_zero() -> vecdb::Result<()> {
            run_take_zero::<V>()
        }
        #[test]
        fn size_hint_consistency() -> vecdb::Result<()> {
            run_size_hint_consistency::<V>()
        }
    }

    #[cfg(feature = "lz4")]
    mod lz4 {
        use super::*;
        use vecdb::LZ4Vec;
        type V = LZ4Vec<usize, i32>;

        #[test]
        fn basic() -> vecdb::Result<()> {
            run_basic::<V>()
        }
        #[test]
        fn nth() -> vecdb::Result<()> {
            run_nth::<V>()
        }
        #[test]
        fn skip() -> vecdb::Result<()> {
            run_skip::<V>()
        }
        #[test]
        fn take() -> vecdb::Result<()> {
            run_take::<V>()
        }
        #[test]
        fn set_position() -> vecdb::Result<()> {
            run_set_position::<V>()
        }
        #[test]
        fn set_end() -> vecdb::Result<()> {
            run_set_end::<V>()
        }
        #[test]
        fn last() -> vecdb::Result<()> {
            run_last::<V>()
        }
        #[test]
        fn last_empty() -> vecdb::Result<()> {
            run_last_empty::<V>()
        }
        #[test]
        fn exact_size() -> vecdb::Result<()> {
            run_exact_size::<V>()
        }
        #[test]
        fn buffer_crossing() -> vecdb::Result<()> {
            run_buffer_crossing::<V>()
        }
        #[test]
        fn multiple_skip_take() -> vecdb::Result<()> {
            run_multiple_skip_take::<V>()
        }
        #[test]
        fn set_position_multiple() -> vecdb::Result<()> {
            run_set_position_multiple::<V>()
        }
        #[test]
        fn nth_beyond_end() -> vecdb::Result<()> {
            run_nth_beyond_end::<V>()
        }
        #[test]
        fn skip_all() -> vecdb::Result<()> {
            run_skip_all::<V>()
        }
        #[test]
        fn take_zero() -> vecdb::Result<()> {
            run_take_zero::<V>()
        }
        #[test]
        fn size_hint_consistency() -> vecdb::Result<()> {
            run_size_hint_consistency::<V>()
        }
    }

    #[cfg(feature = "zstd")]
    mod zstd {
        use super::*;
        use vecdb::ZstdVec;
        type V = ZstdVec<usize, i32>;

        #[test]
        fn basic() -> vecdb::Result<()> {
            run_basic::<V>()
        }
        #[test]
        fn nth() -> vecdb::Result<()> {
            run_nth::<V>()
        }
        #[test]
        fn skip() -> vecdb::Result<()> {
            run_skip::<V>()
        }
        #[test]
        fn take() -> vecdb::Result<()> {
            run_take::<V>()
        }
        #[test]
        fn set_position() -> vecdb::Result<()> {
            run_set_position::<V>()
        }
        #[test]
        fn set_end() -> vecdb::Result<()> {
            run_set_end::<V>()
        }
        #[test]
        fn last() -> vecdb::Result<()> {
            run_last::<V>()
        }
        #[test]
        fn last_empty() -> vecdb::Result<()> {
            run_last_empty::<V>()
        }
        #[test]
        fn exact_size() -> vecdb::Result<()> {
            run_exact_size::<V>()
        }
        #[test]
        fn buffer_crossing() -> vecdb::Result<()> {
            run_buffer_crossing::<V>()
        }
        #[test]
        fn multiple_skip_take() -> vecdb::Result<()> {
            run_multiple_skip_take::<V>()
        }
        #[test]
        fn set_position_multiple() -> vecdb::Result<()> {
            run_set_position_multiple::<V>()
        }
        #[test]
        fn nth_beyond_end() -> vecdb::Result<()> {
            run_nth_beyond_end::<V>()
        }
        #[test]
        fn skip_all() -> vecdb::Result<()> {
            run_skip_all::<V>()
        }
        #[test]
        fn take_zero() -> vecdb::Result<()> {
            run_take_zero::<V>()
        }
        #[test]
        fn size_hint_consistency() -> vecdb::Result<()> {
            run_size_hint_consistency::<V>()
        }
    }

    // ============================================================================
    // EagerVec Tests (wrapping different underlying vec types)
    // ============================================================================

    #[cfg(feature = "zerocopy")]
    mod eager_zerocopy {
        use super::*;
        use vecdb::{EagerVec, ZeroCopyVec};
        type V = EagerVec<ZeroCopyVec<usize, i32>>;

        #[test]
        fn basic() -> vecdb::Result<()> {
            run_basic::<V>()
        }
        #[test]
        fn nth() -> vecdb::Result<()> {
            run_nth::<V>()
        }
        #[test]
        fn skip() -> vecdb::Result<()> {
            run_skip::<V>()
        }
        #[test]
        fn take() -> vecdb::Result<()> {
            run_take::<V>()
        }
        #[test]
        fn set_position() -> vecdb::Result<()> {
            run_set_position::<V>()
        }
        #[test]
        fn set_end() -> vecdb::Result<()> {
            run_set_end::<V>()
        }
        #[test]
        fn last() -> vecdb::Result<()> {
            run_last::<V>()
        }
        #[test]
        fn last_empty() -> vecdb::Result<()> {
            run_last_empty::<V>()
        }
        #[test]
        fn exact_size() -> vecdb::Result<()> {
            run_exact_size::<V>()
        }
        #[test]
        fn buffer_crossing() -> vecdb::Result<()> {
            run_buffer_crossing::<V>()
        }
        #[test]
        fn multiple_skip_take() -> vecdb::Result<()> {
            run_multiple_skip_take::<V>()
        }
        #[test]
        fn set_position_multiple() -> vecdb::Result<()> {
            run_set_position_multiple::<V>()
        }
        #[test]
        fn nth_beyond_end() -> vecdb::Result<()> {
            run_nth_beyond_end::<V>()
        }
        #[test]
        fn skip_all() -> vecdb::Result<()> {
            run_skip_all::<V>()
        }
        #[test]
        fn take_zero() -> vecdb::Result<()> {
            run_take_zero::<V>()
        }
        #[test]
        fn size_hint_consistency() -> vecdb::Result<()> {
            run_size_hint_consistency::<V>()
        }
    }

    #[cfg(feature = "pco")]
    mod eager_pco {
        use super::*;
        use vecdb::{EagerVec, PcoVec};
        type V = EagerVec<PcoVec<usize, i32>>;

        #[test]
        fn basic() -> vecdb::Result<()> {
            run_basic::<V>()
        }
        #[test]
        fn nth() -> vecdb::Result<()> {
            run_nth::<V>()
        }
        #[test]
        fn skip() -> vecdb::Result<()> {
            run_skip::<V>()
        }
        #[test]
        fn take() -> vecdb::Result<()> {
            run_take::<V>()
        }
        #[test]
        fn set_position() -> vecdb::Result<()> {
            run_set_position::<V>()
        }
        #[test]
        fn set_end() -> vecdb::Result<()> {
            run_set_end::<V>()
        }
        #[test]
        fn last() -> vecdb::Result<()> {
            run_last::<V>()
        }
        #[test]
        fn last_empty() -> vecdb::Result<()> {
            run_last_empty::<V>()
        }
        #[test]
        fn exact_size() -> vecdb::Result<()> {
            run_exact_size::<V>()
        }
        #[test]
        fn buffer_crossing() -> vecdb::Result<()> {
            run_buffer_crossing::<V>()
        }
        #[test]
        fn multiple_skip_take() -> vecdb::Result<()> {
            run_multiple_skip_take::<V>()
        }
        #[test]
        fn set_position_multiple() -> vecdb::Result<()> {
            run_set_position_multiple::<V>()
        }
        #[test]
        fn nth_beyond_end() -> vecdb::Result<()> {
            run_nth_beyond_end::<V>()
        }
        #[test]
        fn skip_all() -> vecdb::Result<()> {
            run_skip_all::<V>()
        }
        #[test]
        fn take_zero() -> vecdb::Result<()> {
            run_take_zero::<V>()
        }
        #[test]
        fn size_hint_consistency() -> vecdb::Result<()> {
            run_size_hint_consistency::<V>()
        }
    }
}

// ============================================================================
// Generic Dirty Iterator Tests (stored + pushed data)
// ============================================================================

mod dirty_iter {
    use super::*;

    fn run_only_stored<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        let collected: Vec<i32> = vec.collect();
        assert_eq!(collected.len(), 100);
        assert_eq!(collected[0], 0);
        assert_eq!(collected[99], 99);
        Ok(())
    }

    fn run_only_pushed<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..50 {
            vec.push(i);
        }
        // Don't flush

        let collected: Vec<i32> = vec.collect();
        assert_eq!(collected.len(), 50);
        assert_eq!(collected[0], 0);
        assert_eq!(collected[49], 49);
        Ok(())
    }

    fn run_stored_and_pushed<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..50 {
            vec.push(i);
        }
        vec.write()?;

        for i in 50..100 {
            vec.push(i);
        }

        let collected: Vec<i32> = vec.collect();
        assert_eq!(collected.len(), 100);

        for (i, &val) in collected.iter().enumerate() {
            assert_eq!(val, i as i32);
        }
        Ok(())
    }

    fn run_skip_across_boundary<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..50 {
            vec.push(i);
        }
        vec.write()?;

        for i in 50..100 {
            vec.push(i);
        }

        let collected = vec.collect_range(40, 100);
        assert_eq!(collected.len(), 60);
        assert_eq!(collected[0], 40);
        assert_eq!(collected[59], 99);
        Ok(())
    }

    fn run_take_across_boundary<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..50 {
            vec.push(i);
        }
        vec.write()?;

        for i in 50..100 {
            vec.push(i);
        }

        let collected = vec.collect_range(40, 60);
        assert_eq!(collected.len(), 20);
        assert_eq!(collected[0], 40);
        assert_eq!(collected[19], 59);
        Ok(())
    }

    fn run_nth_across_boundary<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..50 {
            vec.push(i);
        }
        vec.write()?;

        for i in 50..100 {
            vec.push(i);
        }

        // Test reading across stored/pushed boundary
        let vals = vec.collect_range(45, 52);
        assert_eq!(vals[0], 45); // In stored
        assert_eq!(vals[1], 46); // In stored
        assert_eq!(vals[4], 49); // In stored
        assert_eq!(vals[5], 50); // In pushed
        assert_eq!(vals[6], 51); // In pushed
        Ok(())
    }

    fn run_set_position_to_pushed<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..50 {
            vec.push(i);
        }
        vec.write()?;

        for i in 50..100 {
            vec.push(i);
        }

        let vals = vec.collect_range(75, 77);
        assert_eq!(vals[0], 75);
        assert_eq!(vals[1], 76);
        Ok(())
    }

    fn run_last_in_pushed<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..50 {
            vec.push(i);
        }
        vec.write()?;

        for i in 50..100 {
            vec.push(i);
        }

        assert_eq!(vec.collect_last().unwrap(), 99);
        Ok(())
    }

    fn run_last_in_stored<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..100 {
            vec.push(i);
        }
        vec.write()?;

        assert_eq!(vec.collect_last().unwrap(), 99);
        Ok(())
    }

    fn run_exact_size_with_pushed<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..50 {
            vec.push(i);
        }
        vec.write()?;

        for i in 50..75 {
            vec.push(i);
        }

        assert_eq!(vec.len(), 75);
        assert_eq!(vec.collect_range(0, 75).len(), 75);
        assert_eq!(vec.collect_range(1, 75).len(), 74);
        assert_eq!(vec.collect_range(51, 75).len(), 24); // Cross boundary
        Ok(())
    }

    fn run_large_dataset_boundary<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        // Large stored portion
        for i in 0..10000 {
            vec.push(i);
        }
        vec.write()?;

        // Small pushed portion
        for i in 10000..10100 {
            vec.push(i);
        }

        let collected: Vec<i32> = vec.collect();
        assert_eq!(collected.len(), 10100);

        for (i, &val) in collected.iter().enumerate() {
            assert_eq!(val, i as i32);
        }
        Ok(())
    }

    fn run_skip_take_complex<V>() -> vecdb::Result<()>
    where
        V: StoredVec<I = usize, T = i32>,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..8000 {
            vec.push(i);
        }
        vec.write()?;

        for i in 8000..12000 {
            vec.push(i);
        }

        // skip(7000).take(3000).skip(500).take(1000) => collect_range(7500, 8500)
        let collected = vec.collect_range(7500, 8500);

        assert_eq!(collected.len(), 1000);
        assert_eq!(collected[0], 7500);
        assert_eq!(collected[999], 8499);
        Ok(())
    }

    // ============================================================================
    // Test instantiation for each vec type
    // ============================================================================

    mod bytes {
        use super::*;
        use vecdb::BytesVec;
        type V = BytesVec<usize, i32>;

        #[test]
        fn only_stored() -> vecdb::Result<()> {
            run_only_stored::<V>()
        }
        #[test]
        fn only_pushed() -> vecdb::Result<()> {
            run_only_pushed::<V>()
        }
        #[test]
        fn stored_and_pushed() -> vecdb::Result<()> {
            run_stored_and_pushed::<V>()
        }
        #[test]
        fn skip_across_boundary() -> vecdb::Result<()> {
            run_skip_across_boundary::<V>()
        }
        #[test]
        fn take_across_boundary() -> vecdb::Result<()> {
            run_take_across_boundary::<V>()
        }
        #[test]
        fn nth_across_boundary() -> vecdb::Result<()> {
            run_nth_across_boundary::<V>()
        }
        #[test]
        fn set_position_to_pushed() -> vecdb::Result<()> {
            run_set_position_to_pushed::<V>()
        }
        #[test]
        fn last_in_pushed() -> vecdb::Result<()> {
            run_last_in_pushed::<V>()
        }
        #[test]
        fn last_in_stored() -> vecdb::Result<()> {
            run_last_in_stored::<V>()
        }
        #[test]
        fn exact_size_with_pushed() -> vecdb::Result<()> {
            run_exact_size_with_pushed::<V>()
        }
        #[test]
        fn large_dataset_boundary() -> vecdb::Result<()> {
            run_large_dataset_boundary::<V>()
        }
        #[test]
        fn skip_take_complex() -> vecdb::Result<()> {
            run_skip_take_complex::<V>()
        }
    }

    #[cfg(feature = "zerocopy")]
    mod zerocopy {
        use super::*;
        use vecdb::ZeroCopyVec;
        type V = ZeroCopyVec<usize, i32>;

        #[test]
        fn only_stored() -> vecdb::Result<()> {
            run_only_stored::<V>()
        }
        #[test]
        fn only_pushed() -> vecdb::Result<()> {
            run_only_pushed::<V>()
        }
        #[test]
        fn stored_and_pushed() -> vecdb::Result<()> {
            run_stored_and_pushed::<V>()
        }
        #[test]
        fn skip_across_boundary() -> vecdb::Result<()> {
            run_skip_across_boundary::<V>()
        }
        #[test]
        fn take_across_boundary() -> vecdb::Result<()> {
            run_take_across_boundary::<V>()
        }
        #[test]
        fn nth_across_boundary() -> vecdb::Result<()> {
            run_nth_across_boundary::<V>()
        }
        #[test]
        fn set_position_to_pushed() -> vecdb::Result<()> {
            run_set_position_to_pushed::<V>()
        }
        #[test]
        fn last_in_pushed() -> vecdb::Result<()> {
            run_last_in_pushed::<V>()
        }
        #[test]
        fn last_in_stored() -> vecdb::Result<()> {
            run_last_in_stored::<V>()
        }
        #[test]
        fn exact_size_with_pushed() -> vecdb::Result<()> {
            run_exact_size_with_pushed::<V>()
        }
        #[test]
        fn large_dataset_boundary() -> vecdb::Result<()> {
            run_large_dataset_boundary::<V>()
        }
        #[test]
        fn skip_take_complex() -> vecdb::Result<()> {
            run_skip_take_complex::<V>()
        }
    }

    #[cfg(feature = "pco")]
    mod pco {
        use super::*;
        use vecdb::PcoVec;
        type V = PcoVec<usize, i32>;

        #[test]
        fn only_stored() -> vecdb::Result<()> {
            run_only_stored::<V>()
        }
        #[test]
        fn only_pushed() -> vecdb::Result<()> {
            run_only_pushed::<V>()
        }
        #[test]
        fn stored_and_pushed() -> vecdb::Result<()> {
            run_stored_and_pushed::<V>()
        }
        #[test]
        fn skip_across_boundary() -> vecdb::Result<()> {
            run_skip_across_boundary::<V>()
        }
        #[test]
        fn take_across_boundary() -> vecdb::Result<()> {
            run_take_across_boundary::<V>()
        }
        #[test]
        fn nth_across_boundary() -> vecdb::Result<()> {
            run_nth_across_boundary::<V>()
        }
        #[test]
        fn set_position_to_pushed() -> vecdb::Result<()> {
            run_set_position_to_pushed::<V>()
        }
        #[test]
        fn last_in_pushed() -> vecdb::Result<()> {
            run_last_in_pushed::<V>()
        }
        #[test]
        fn last_in_stored() -> vecdb::Result<()> {
            run_last_in_stored::<V>()
        }
        #[test]
        fn exact_size_with_pushed() -> vecdb::Result<()> {
            run_exact_size_with_pushed::<V>()
        }
        #[test]
        fn large_dataset_boundary() -> vecdb::Result<()> {
            run_large_dataset_boundary::<V>()
        }
        #[test]
        fn skip_take_complex() -> vecdb::Result<()> {
            run_skip_take_complex::<V>()
        }
    }

    #[cfg(feature = "lz4")]
    mod lz4 {
        use super::*;
        use vecdb::LZ4Vec;
        type V = LZ4Vec<usize, i32>;

        #[test]
        fn only_stored() -> vecdb::Result<()> {
            run_only_stored::<V>()
        }
        #[test]
        fn only_pushed() -> vecdb::Result<()> {
            run_only_pushed::<V>()
        }
        #[test]
        fn stored_and_pushed() -> vecdb::Result<()> {
            run_stored_and_pushed::<V>()
        }
        #[test]
        fn skip_across_boundary() -> vecdb::Result<()> {
            run_skip_across_boundary::<V>()
        }
        #[test]
        fn take_across_boundary() -> vecdb::Result<()> {
            run_take_across_boundary::<V>()
        }
        #[test]
        fn nth_across_boundary() -> vecdb::Result<()> {
            run_nth_across_boundary::<V>()
        }
        #[test]
        fn set_position_to_pushed() -> vecdb::Result<()> {
            run_set_position_to_pushed::<V>()
        }
        #[test]
        fn last_in_pushed() -> vecdb::Result<()> {
            run_last_in_pushed::<V>()
        }
        #[test]
        fn last_in_stored() -> vecdb::Result<()> {
            run_last_in_stored::<V>()
        }
        #[test]
        fn exact_size_with_pushed() -> vecdb::Result<()> {
            run_exact_size_with_pushed::<V>()
        }
        #[test]
        fn large_dataset_boundary() -> vecdb::Result<()> {
            run_large_dataset_boundary::<V>()
        }
        #[test]
        fn skip_take_complex() -> vecdb::Result<()> {
            run_skip_take_complex::<V>()
        }
    }

    #[cfg(feature = "zstd")]
    mod zstd {
        use super::*;
        use vecdb::ZstdVec;
        type V = ZstdVec<usize, i32>;

        #[test]
        fn only_stored() -> vecdb::Result<()> {
            run_only_stored::<V>()
        }
        #[test]
        fn only_pushed() -> vecdb::Result<()> {
            run_only_pushed::<V>()
        }
        #[test]
        fn stored_and_pushed() -> vecdb::Result<()> {
            run_stored_and_pushed::<V>()
        }
        #[test]
        fn skip_across_boundary() -> vecdb::Result<()> {
            run_skip_across_boundary::<V>()
        }
        #[test]
        fn take_across_boundary() -> vecdb::Result<()> {
            run_take_across_boundary::<V>()
        }
        #[test]
        fn nth_across_boundary() -> vecdb::Result<()> {
            run_nth_across_boundary::<V>()
        }
        #[test]
        fn set_position_to_pushed() -> vecdb::Result<()> {
            run_set_position_to_pushed::<V>()
        }
        #[test]
        fn last_in_pushed() -> vecdb::Result<()> {
            run_last_in_pushed::<V>()
        }
        #[test]
        fn last_in_stored() -> vecdb::Result<()> {
            run_last_in_stored::<V>()
        }
        #[test]
        fn exact_size_with_pushed() -> vecdb::Result<()> {
            run_exact_size_with_pushed::<V>()
        }
        #[test]
        fn large_dataset_boundary() -> vecdb::Result<()> {
            run_large_dataset_boundary::<V>()
        }
        #[test]
        fn skip_take_complex() -> vecdb::Result<()> {
            run_skip_take_complex::<V>()
        }
    }

    // ============================================================================
    // EagerVec Tests (wrapping different underlying vec types)
    // ============================================================================

    #[cfg(feature = "zerocopy")]
    mod eager_zerocopy {
        use super::*;
        use vecdb::{EagerVec, ZeroCopyVec};
        type V = EagerVec<ZeroCopyVec<usize, i32>>;

        #[test]
        fn only_stored() -> vecdb::Result<()> {
            run_only_stored::<V>()
        }
        #[test]
        fn only_pushed() -> vecdb::Result<()> {
            run_only_pushed::<V>()
        }
        #[test]
        fn stored_and_pushed() -> vecdb::Result<()> {
            run_stored_and_pushed::<V>()
        }
        #[test]
        fn skip_across_boundary() -> vecdb::Result<()> {
            run_skip_across_boundary::<V>()
        }
        #[test]
        fn take_across_boundary() -> vecdb::Result<()> {
            run_take_across_boundary::<V>()
        }
        #[test]
        fn nth_across_boundary() -> vecdb::Result<()> {
            run_nth_across_boundary::<V>()
        }
        #[test]
        fn set_position_to_pushed() -> vecdb::Result<()> {
            run_set_position_to_pushed::<V>()
        }
        #[test]
        fn last_in_pushed() -> vecdb::Result<()> {
            run_last_in_pushed::<V>()
        }
        #[test]
        fn last_in_stored() -> vecdb::Result<()> {
            run_last_in_stored::<V>()
        }
        #[test]
        fn exact_size_with_pushed() -> vecdb::Result<()> {
            run_exact_size_with_pushed::<V>()
        }
        #[test]
        fn large_dataset_boundary() -> vecdb::Result<()> {
            run_large_dataset_boundary::<V>()
        }
        #[test]
        fn skip_take_complex() -> vecdb::Result<()> {
            run_skip_take_complex::<V>()
        }
    }

    #[cfg(feature = "pco")]
    mod eager_pco {
        use super::*;
        use vecdb::{EagerVec, PcoVec};
        type V = EagerVec<PcoVec<usize, i32>>;

        #[test]
        fn only_stored() -> vecdb::Result<()> {
            run_only_stored::<V>()
        }
        #[test]
        fn only_pushed() -> vecdb::Result<()> {
            run_only_pushed::<V>()
        }
        #[test]
        fn stored_and_pushed() -> vecdb::Result<()> {
            run_stored_and_pushed::<V>()
        }
        #[test]
        fn skip_across_boundary() -> vecdb::Result<()> {
            run_skip_across_boundary::<V>()
        }
        #[test]
        fn take_across_boundary() -> vecdb::Result<()> {
            run_take_across_boundary::<V>()
        }
        #[test]
        fn nth_across_boundary() -> vecdb::Result<()> {
            run_nth_across_boundary::<V>()
        }
        #[test]
        fn set_position_to_pushed() -> vecdb::Result<()> {
            run_set_position_to_pushed::<V>()
        }
        #[test]
        fn last_in_pushed() -> vecdb::Result<()> {
            run_last_in_pushed::<V>()
        }
        #[test]
        fn last_in_stored() -> vecdb::Result<()> {
            run_last_in_stored::<V>()
        }
        #[test]
        fn exact_size_with_pushed() -> vecdb::Result<()> {
            run_exact_size_with_pushed::<V>()
        }
        #[test]
        fn large_dataset_boundary() -> vecdb::Result<()> {
            run_large_dataset_boundary::<V>()
        }
        #[test]
        fn skip_take_complex() -> vecdb::Result<()> {
            run_skip_take_complex::<V>()
        }
    }
}

// ============================================================================
// Mutable Raw-Vector Tests (holes and updates)
// ============================================================================

mod raw_features {
    use super::*;
    use vecdb::{BytesVec, MutableVec};

    #[cfg(feature = "zerocopy")]
    use vecdb::ZeroCopyVec;

    // Generic test functions for MutableVec over raw vecs

    fn run_iter_skips_holes<V>() -> vecdb::Result<()>
    where
        V: RawVecOps,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..10 {
            vec.push(i);
        }
        vec.write()?;

        // Delete some values (create holes)
        vec.delete_at(3);
        vec.delete_at(5);
        vec.delete_at(7);

        let collected: Vec<i32> = vec.collect();
        // Should skip holes: 0,1,2,4,6,8,9
        assert_eq!(collected, vec![0, 1, 2, 4, 6, 8, 9]);
        Ok(())
    }

    fn run_iter_with_updates<V>() -> vecdb::Result<()>
    where
        V: RawVecOps,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..10 {
            vec.push(i);
        }
        vec.write()?;

        // Update some values
        vec.update_at(2, 200)?;
        vec.update_at(5, 500)?;
        vec.update_at(8, 800)?;

        let collected: Vec<i32> = vec.collect();
        assert_eq!(collected, vec![0, 1, 200, 3, 4, 500, 6, 7, 800, 9]);
        Ok(())
    }

    fn run_iter_with_holes_and_updates<V>() -> vecdb::Result<()>
    where
        V: RawVecOps,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..10 {
            vec.push(i);
        }
        vec.write()?;

        // Create holes and updates
        vec.delete_at(1);
        vec.delete_at(3);
        vec.update_at(2, 200)?;
        vec.update_at(5, 500)?;

        let collected: Vec<i32> = vec.collect();
        // Should be: 0, (skip 1), 200, (skip 3), 4, 500, 6, 7, 8, 9
        assert_eq!(collected, vec![0, 200, 4, 500, 6, 7, 8, 9]);
        Ok(())
    }

    fn run_iter_holes_and_pushed<V>() -> vecdb::Result<()>
    where
        V: RawVecOps,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..5 {
            vec.push(i);
        }
        vec.write()?;

        // Create holes in stored data
        vec.delete_at(1);
        vec.delete_at(3);

        // Push more data
        for i in 5..10 {
            vec.push(i);
        }

        let collected: Vec<i32> = vec.collect();
        // Should be: 0, (skip 1), 2, (skip 3), 4, 5, 6, 7, 8, 9
        assert_eq!(collected, vec![0, 2, 4, 5, 6, 7, 8, 9]);
        Ok(())
    }

    fn run_iter_updates_and_pushed<V>() -> vecdb::Result<()>
    where
        V: RawVecOps,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..5 {
            vec.push(i);
        }
        vec.write()?;

        // Update some stored values
        vec.update_at(1, 100)?;
        vec.update_at(3, 300)?;

        // Push more data
        for i in 5..10 {
            vec.push(i);
        }

        let collected: Vec<i32> = vec.collect();
        assert_eq!(collected, vec![0, 100, 2, 300, 4, 5, 6, 7, 8, 9]);
        Ok(())
    }

    fn run_iter_skip_over_holes<V>() -> vecdb::Result<()>
    where
        V: RawVecOps,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..20 {
            vec.push(i);
        }
        vec.write()?;

        // Create holes at indices 5, 6, 7
        vec.delete_at(5);
        vec.delete_at(6);
        vec.delete_at(7);

        // Skip past the holes — collect skips holes automatically
        let collected: Vec<i32> = vec.collect();
        // Should be: 0, 1, 2, 3, 4, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19
        // (holes at 5,6,7 are skipped)
        assert_eq!(collected[5..10], [8, 9, 10, 11, 12]);
        Ok(())
    }

    fn run_fill_holes<V>() -> vecdb::Result<()>
    where
        V: RawVecOps,
    {
        let (db, _temp) = setup_db()?;
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;

        for i in 0..10 {
            vec.push(i);
        }
        vec.write()?;

        // Create holes
        vec.delete_at(2);
        vec.delete_at(5);

        // Fill first hole
        let idx = vec.fill_first_hole_or_push(999)?;
        assert_eq!(idx, 2);

        let collected: Vec<i32> = vec.collect();
        // 0,1,999,3,4,(skip 5),6,7,8,9
        assert_eq!(collected, vec![0, 1, 999, 3, 4, 6, 7, 8, 9]);
        Ok(())
    }

    // Helper trait for mutable raw-vector operations
    pub trait RawVecOps: StoredVec<I = usize, T = i32> {
        fn delete_at(&mut self, index: usize);
        fn update_at(&mut self, index: usize, value: i32) -> vecdb::Result<()>;
        fn fill_first_hole_or_push(&mut self, value: i32) -> vecdb::Result<usize>;
    }

    impl RawVecOps for MutableVec<BytesVec<usize, i32>> {
        fn delete_at(&mut self, index: usize) {
            MutableVec::<BytesVec<usize, i32>>::delete_at(self, index)
        }
        fn update_at(&mut self, index: usize, value: i32) -> vecdb::Result<()> {
            MutableVec::<BytesVec<usize, i32>>::update_at(self, index, value)
        }
        fn fill_first_hole_or_push(&mut self, value: i32) -> vecdb::Result<usize> {
            MutableVec::<BytesVec<usize, i32>>::fill_first_hole_or_push(self, value)
        }
    }

    #[cfg(feature = "zerocopy")]
    impl RawVecOps for MutableVec<ZeroCopyVec<usize, i32>> {
        fn delete_at(&mut self, index: usize) {
            MutableVec::<ZeroCopyVec<usize, i32>>::delete_at(self, index)
        }
        fn update_at(&mut self, index: usize, value: i32) -> vecdb::Result<()> {
            MutableVec::<ZeroCopyVec<usize, i32>>::update_at(self, index, value)
        }
        fn fill_first_hole_or_push(&mut self, value: i32) -> vecdb::Result<usize> {
            MutableVec::<ZeroCopyVec<usize, i32>>::fill_first_hole_or_push(self, value)
        }
    }

    // ============================================================================
    // BytesVec Tests
    // ============================================================================

    mod bytes {
        use super::*;

        #[test]
        fn iter_skips_holes() -> vecdb::Result<()> {
            run_iter_skips_holes::<MutableVec<BytesVec<usize, i32>>>()
        }
        #[test]
        fn iter_with_updates() -> vecdb::Result<()> {
            run_iter_with_updates::<MutableVec<BytesVec<usize, i32>>>()
        }
        #[test]
        fn iter_with_holes_and_updates() -> vecdb::Result<()> {
            run_iter_with_holes_and_updates::<MutableVec<BytesVec<usize, i32>>>()
        }
        #[test]
        fn iter_holes_and_pushed() -> vecdb::Result<()> {
            run_iter_holes_and_pushed::<MutableVec<BytesVec<usize, i32>>>()
        }
        #[test]
        fn iter_updates_and_pushed() -> vecdb::Result<()> {
            run_iter_updates_and_pushed::<MutableVec<BytesVec<usize, i32>>>()
        }
        #[test]
        fn iter_skip_over_holes() -> vecdb::Result<()> {
            run_iter_skip_over_holes::<MutableVec<BytesVec<usize, i32>>>()
        }
        #[test]
        fn fill_holes() -> vecdb::Result<()> {
            run_fill_holes::<MutableVec<BytesVec<usize, i32>>>()
        }
    }

    // ============================================================================
    // ZeroCopyVec Tests
    // ============================================================================

    #[cfg(feature = "zerocopy")]
    mod zerocopy {
        use super::*;

        #[test]
        fn iter_skips_holes() -> vecdb::Result<()> {
            run_iter_skips_holes::<MutableVec<ZeroCopyVec<usize, i32>>>()
        }
        #[test]
        fn iter_with_updates() -> vecdb::Result<()> {
            run_iter_with_updates::<MutableVec<ZeroCopyVec<usize, i32>>>()
        }
        #[test]
        fn iter_with_holes_and_updates() -> vecdb::Result<()> {
            run_iter_with_holes_and_updates::<MutableVec<ZeroCopyVec<usize, i32>>>()
        }
        #[test]
        fn iter_holes_and_pushed() -> vecdb::Result<()> {
            run_iter_holes_and_pushed::<MutableVec<ZeroCopyVec<usize, i32>>>()
        }
        #[test]
        fn iter_updates_and_pushed() -> vecdb::Result<()> {
            run_iter_updates_and_pushed::<MutableVec<ZeroCopyVec<usize, i32>>>()
        }
        #[test]
        fn iter_skip_over_holes() -> vecdb::Result<()> {
            run_iter_skip_over_holes::<MutableVec<ZeroCopyVec<usize, i32>>>()
        }
        #[test]
        fn fill_holes() -> vecdb::Result<()> {
            run_fill_holes::<MutableVec<ZeroCopyVec<usize, i32>>>()
        }
    }
}
