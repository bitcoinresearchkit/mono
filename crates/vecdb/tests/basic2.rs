//! Generic basic operations tests for all vec types.
//!
//! These tests run against any type implementing `StoredVec`, ensuring
//! consistent behavior across BytesVec, ZeroCopyVec, PcoVec, LZ4Vec, ZstdVec, and EagerVec.

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{Stamp, StoredVec, Version};

// ============================================================================
// Test Setup
// ============================================================================

fn setup_db() -> vecdb::Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

// ============================================================================
// Generic Basic Operations Tests
// ============================================================================

fn run_push_write_read<V>() -> vecdb::Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;

    // Push values
    for i in 0..21_u32 {
        vec.push(i);
    }

    // Read via get before write
    assert_eq!(vec.collect_range(0, 1), vec![0]);
    assert_eq!(vec.collect_range(1, 2), vec![1]);
    assert_eq!(vec.collect_range(2, 3), vec![2]);
    assert_eq!(vec.collect_range(20, 21), vec![20]);
    assert!(vec.collect_range(21, 22).is_empty());

    // Write to storage
    vec.write()?;

    // Verify stamp
    assert_eq!(vec.header().stamp(), Stamp::new(0));

    Ok(())
}

fn run_stamp_management<V>() -> vecdb::Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;

    for i in 0..21_u32 {
        vec.push(i);
    }
    vec.write()?;

    // Reimport and update stamp
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;
    vec.mut_header().update_stamp(Stamp::new(100));

    assert_eq!(vec.header().stamp(), Stamp::new(100));

    // Verify data still readable
    assert_eq!(vec.collect_range(0, 1), vec![0]);
    assert_eq!(vec.collect_range(1, 2), vec![1]);
    assert_eq!(vec.collect_range(20, 21), vec![20]);

    Ok(())
}

fn run_length_tracking<V>() -> vecdb::Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;

    // Push and write
    for i in 0..21_u32 {
        vec.push(i);
    }
    vec.write()?;

    // Reimport and verify
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;
    assert_eq!(vec.stored_len(), 21);
    assert_eq!(vec.pushed_len(), 0);
    assert_eq!(vec.len(), 21);

    // Push more
    vec.push(21);
    vec.push(22);

    assert_eq!(vec.stored_len(), 21);
    assert_eq!(vec.pushed_len(), 2);
    assert_eq!(vec.len(), 23);

    // Read across stored/pushed boundary
    assert_eq!(vec.collect_range(20, 21), vec![20]);
    assert_eq!(vec.collect_range(21, 22), vec![21]);
    assert_eq!(vec.collect_range(22, 23), vec![22]);
    assert!(vec.collect_range(23, 24).is_empty());

    // Write and verify persistence
    vec.write()?;

    let vec = V::forced_import(&db, "test", Version::ONE)?;
    assert_eq!(vec.stored_len(), 23);
    assert_eq!(vec.pushed_len(), 0);
    assert_eq!(vec.len(), 23);

    Ok(())
}

fn run_truncate<V>() -> vecdb::Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;

    // Push and write
    for i in 0..23_u32 {
        vec.push(i);
    }
    vec.write()?;

    // Reimport and truncate
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;
    vec.truncate_if_needed(14)?;

    assert_eq!(vec.stored_len(), 14);
    assert_eq!(vec.pushed_len(), 0);
    assert_eq!(vec.len(), 14);

    // Verify truncated data
    assert_eq!(vec.collect_range(0, 1), vec![0]);
    assert_eq!(vec.collect_range(5, 6), vec![5]);
    assert_eq!(vec.collect_range(13, 14), vec![13]);
    assert!(vec.collect_range(14, 15).is_empty());
    assert!(vec.collect_range(20, 21).is_empty());

    Ok(())
}

fn run_collect_signed_range<V>() -> vecdb::Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;

    for i in 0..15_u32 {
        vec.push(i);
    }
    vec.write()?;

    // Test negative range (last 5 elements)
    assert_eq!(
        vec.collect_signed_range(Some(-5), None),
        vec![10, 11, 12, 13, 14]
    );

    // Test positive range
    assert_eq!(
        vec.collect_signed_range(Some(5), Some(10)),
        vec![5, 6, 7, 8, 9]
    );

    Ok(())
}

fn run_iter_last<V>() -> vecdb::Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;

    for i in 0..15_u32 {
        vec.push(i);
    }
    vec.write()?;

    // Push one more without writing
    vec.push(15);

    let last = vec.collect_range(vec.len() - 1, vec.len());
    assert_eq!(last[0], 15);

    Ok(())
}

fn run_reset<V>() -> vecdb::Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;

    for i in 0..15_u32 {
        vec.push(i);
    }
    vec.write()?;

    // Reset
    vec.reset()?;

    assert_eq!(vec.pushed_len(), 0);
    assert_eq!(vec.stored_len(), 0);
    assert_eq!(vec.len(), 0);

    // Push new data after reset
    for i in 0..21_u32 {
        vec.push(i);
    }

    assert_eq!(vec.pushed_len(), 21);
    assert_eq!(vec.stored_len(), 0);
    assert_eq!(vec.len(), 21);

    // Verify new data
    assert_eq!(vec.collect_range(0, 1), vec![0]);
    assert_eq!(vec.collect_range(20, 21), vec![20]);
    assert!(vec.collect_range(21, 22).is_empty());

    Ok(())
}

fn run_collect<V>() -> vecdb::Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;

    for i in 0..10_u32 {
        vec.push(i);
    }
    vec.write()?;

    assert_eq!(vec.collect(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    Ok(())
}

fn run_persistence_across_reopen<V>() -> vecdb::Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;

    // Write data
    {
        let mut vec = V::forced_import(&db, "test", Version::ONE)?;
        for i in 0..100_u32 {
            vec.push(i);
        }
        vec.write()?;
    }

    // Reopen and verify
    {
        let vec = V::forced_import(&db, "test", Version::ONE)?;
        assert_eq!(vec.len(), 100);
        assert_eq!(vec.collect().len(), 100);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(50, 51), vec![50]);
        assert_eq!(vec.collect_range(99, 100), vec![99]);
    }

    Ok(())
}

// ============================================================================
// Test instantiation for each vec type
// ============================================================================

mod bytes {
    use super::*;
    use vecdb::BytesVec;
    type V = BytesVec<usize, u32>;

    #[test]
    fn push_write_read() -> vecdb::Result<()> {
        run_push_write_read::<V>()
    }
    #[test]
    fn stamp_management() -> vecdb::Result<()> {
        run_stamp_management::<V>()
    }
    #[test]
    fn length_tracking() -> vecdb::Result<()> {
        run_length_tracking::<V>()
    }
    #[test]
    fn truncate() -> vecdb::Result<()> {
        run_truncate::<V>()
    }
    #[test]
    fn collect_signed_range() -> vecdb::Result<()> {
        run_collect_signed_range::<V>()
    }
    #[test]
    fn iter_last() -> vecdb::Result<()> {
        run_iter_last::<V>()
    }
    #[test]
    fn reset() -> vecdb::Result<()> {
        run_reset::<V>()
    }
    #[test]
    fn collect() -> vecdb::Result<()> {
        run_collect::<V>()
    }
    #[test]
    fn persistence_across_reopen() -> vecdb::Result<()> {
        run_persistence_across_reopen::<V>()
    }
}

#[cfg(feature = "zerocopy")]
mod zerocopy {
    use super::*;
    use vecdb::ZeroCopyVec;
    type V = ZeroCopyVec<usize, u32>;

    #[test]
    fn push_write_read() -> vecdb::Result<()> {
        run_push_write_read::<V>()
    }
    #[test]
    fn stamp_management() -> vecdb::Result<()> {
        run_stamp_management::<V>()
    }
    #[test]
    fn length_tracking() -> vecdb::Result<()> {
        run_length_tracking::<V>()
    }
    #[test]
    fn truncate() -> vecdb::Result<()> {
        run_truncate::<V>()
    }
    #[test]
    fn collect_signed_range() -> vecdb::Result<()> {
        run_collect_signed_range::<V>()
    }
    #[test]
    fn iter_last() -> vecdb::Result<()> {
        run_iter_last::<V>()
    }
    #[test]
    fn reset() -> vecdb::Result<()> {
        run_reset::<V>()
    }
    #[test]
    fn collect() -> vecdb::Result<()> {
        run_collect::<V>()
    }
    #[test]
    fn persistence_across_reopen() -> vecdb::Result<()> {
        run_persistence_across_reopen::<V>()
    }
}

#[cfg(feature = "pco")]
mod pco {
    use super::*;
    use vecdb::PcoVec;
    type V = PcoVec<usize, u32>;

    #[test]
    fn push_write_read() -> vecdb::Result<()> {
        run_push_write_read::<V>()
    }
    #[test]
    fn stamp_management() -> vecdb::Result<()> {
        run_stamp_management::<V>()
    }
    #[test]
    fn length_tracking() -> vecdb::Result<()> {
        run_length_tracking::<V>()
    }
    #[test]
    fn truncate() -> vecdb::Result<()> {
        run_truncate::<V>()
    }
    #[test]
    fn collect_signed_range() -> vecdb::Result<()> {
        run_collect_signed_range::<V>()
    }
    #[test]
    fn iter_last() -> vecdb::Result<()> {
        run_iter_last::<V>()
    }
    #[test]
    fn reset() -> vecdb::Result<()> {
        run_reset::<V>()
    }
    #[test]
    fn collect() -> vecdb::Result<()> {
        run_collect::<V>()
    }
    #[test]
    fn persistence_across_reopen() -> vecdb::Result<()> {
        run_persistence_across_reopen::<V>()
    }
}

#[cfg(feature = "lz4")]
mod lz4 {
    use super::*;
    use vecdb::LZ4Vec;
    type V = LZ4Vec<usize, u32>;

    #[test]
    fn push_write_read() -> vecdb::Result<()> {
        run_push_write_read::<V>()
    }
    #[test]
    fn stamp_management() -> vecdb::Result<()> {
        run_stamp_management::<V>()
    }
    #[test]
    fn length_tracking() -> vecdb::Result<()> {
        run_length_tracking::<V>()
    }
    #[test]
    fn truncate() -> vecdb::Result<()> {
        run_truncate::<V>()
    }
    #[test]
    fn collect_signed_range() -> vecdb::Result<()> {
        run_collect_signed_range::<V>()
    }
    #[test]
    fn iter_last() -> vecdb::Result<()> {
        run_iter_last::<V>()
    }
    #[test]
    fn reset() -> vecdb::Result<()> {
        run_reset::<V>()
    }
    #[test]
    fn collect() -> vecdb::Result<()> {
        run_collect::<V>()
    }
    #[test]
    fn persistence_across_reopen() -> vecdb::Result<()> {
        run_persistence_across_reopen::<V>()
    }
}

#[cfg(feature = "zstd")]
mod zstd {
    use super::*;
    use vecdb::ZstdVec;
    type V = ZstdVec<usize, u32>;

    #[test]
    fn push_write_read() -> vecdb::Result<()> {
        run_push_write_read::<V>()
    }
    #[test]
    fn stamp_management() -> vecdb::Result<()> {
        run_stamp_management::<V>()
    }
    #[test]
    fn length_tracking() -> vecdb::Result<()> {
        run_length_tracking::<V>()
    }
    #[test]
    fn truncate() -> vecdb::Result<()> {
        run_truncate::<V>()
    }
    #[test]
    fn collect_signed_range() -> vecdb::Result<()> {
        run_collect_signed_range::<V>()
    }
    #[test]
    fn iter_last() -> vecdb::Result<()> {
        run_iter_last::<V>()
    }
    #[test]
    fn reset() -> vecdb::Result<()> {
        run_reset::<V>()
    }
    #[test]
    fn collect() -> vecdb::Result<()> {
        run_collect::<V>()
    }
    #[test]
    fn persistence_across_reopen() -> vecdb::Result<()> {
        run_persistence_across_reopen::<V>()
    }
}

#[cfg(feature = "zerocopy")]
mod eager_zerocopy {
    use super::*;
    use vecdb::{EagerVec, ZeroCopyVec};
    type V = EagerVec<ZeroCopyVec<usize, u32>>;

    #[test]
    fn push_write_read() -> vecdb::Result<()> {
        run_push_write_read::<V>()
    }
    #[test]
    fn stamp_management() -> vecdb::Result<()> {
        run_stamp_management::<V>()
    }
    #[test]
    fn length_tracking() -> vecdb::Result<()> {
        run_length_tracking::<V>()
    }
    #[test]
    fn truncate() -> vecdb::Result<()> {
        run_truncate::<V>()
    }
    #[test]
    fn collect_signed_range() -> vecdb::Result<()> {
        run_collect_signed_range::<V>()
    }
    #[test]
    fn iter_last() -> vecdb::Result<()> {
        run_iter_last::<V>()
    }
    #[test]
    fn reset() -> vecdb::Result<()> {
        run_reset::<V>()
    }
    #[test]
    fn collect() -> vecdb::Result<()> {
        run_collect::<V>()
    }
    #[test]
    fn persistence_across_reopen() -> vecdb::Result<()> {
        run_persistence_across_reopen::<V>()
    }
}

#[cfg(feature = "pco")]
mod eager_pco {
    use super::*;
    use vecdb::{EagerVec, PcoVec};
    type V = EagerVec<PcoVec<usize, u32>>;

    #[test]
    fn push_write_read() -> vecdb::Result<()> {
        run_push_write_read::<V>()
    }
    #[test]
    fn stamp_management() -> vecdb::Result<()> {
        run_stamp_management::<V>()
    }
    #[test]
    fn length_tracking() -> vecdb::Result<()> {
        run_length_tracking::<V>()
    }
    #[test]
    fn truncate() -> vecdb::Result<()> {
        run_truncate::<V>()
    }
    #[test]
    fn collect_signed_range() -> vecdb::Result<()> {
        run_collect_signed_range::<V>()
    }
    #[test]
    fn iter_last() -> vecdb::Result<()> {
        run_iter_last::<V>()
    }
    #[test]
    fn reset() -> vecdb::Result<()> {
        run_reset::<V>()
    }
    #[test]
    fn collect() -> vecdb::Result<()> {
        run_collect::<V>()
    }
    #[test]
    fn persistence_across_reopen() -> vecdb::Result<()> {
        run_persistence_across_reopen::<V>()
    }
}
