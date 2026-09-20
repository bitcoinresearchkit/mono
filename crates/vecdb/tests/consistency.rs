//! Generic consistency tests for all vec types.

use vecdb::BytesVec;
#[cfg(feature = "lz4")]
use vecdb::LZ4Vec;
#[cfg(feature = "pco")]
use vecdb::PcoVec;
#[cfg(feature = "zerocopy")]
use vecdb::ZeroCopyVec;
#[cfg(feature = "zstd")]
use vecdb::ZstdVec;

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{AnyStoredVec, EagerVec, ImportableVec, ReadableVec, StoredVec, Version, WritableVec};

/// Bulk and scalar reads agree after an initial write and repeated appends.
fn run_immediate_read_after_write<V>()
where
    V: StoredVec<I = usize, T = u64>,
{
    let temp_dir = TempDir::new().unwrap();
    let db = Database::open(&temp_dir.path().join("test.db")).unwrap();

    let mut vec: EagerVec<V> = EagerVec::forced_import(&db, "test_vec", Version::ONE).unwrap();

    let mut expected = Vec::new();
    for batch in 0..=10 {
        let start = expected.len();
        let end = start + if batch == 0 { 1000 } else { 100 };
        for i in start..end {
            let value = i as u64 * 100;
            vec.push(value);
            expected.push(value);
        }
        vec.flush().unwrap();

        for i in start..end {
            assert_eq!(vec.collect_one(i), Some(i as u64 * 100), "index {i}");
        }
        assert_eq!(vec.collect(), expected, "batch {batch}");
    }
}

#[test]
fn bytes() {
    run_immediate_read_after_write::<BytesVec<usize, u64>>();
}

#[cfg(feature = "zerocopy")]
#[test]
fn zerocopy() {
    run_immediate_read_after_write::<ZeroCopyVec<usize, u64>>();
}

#[cfg(feature = "pco")]
#[test]
fn pco() {
    run_immediate_read_after_write::<PcoVec<usize, u64>>();
}

#[cfg(feature = "lz4")]
#[test]
fn lz4() {
    run_immediate_read_after_write::<LZ4Vec<usize, u64>>();
}

#[cfg(feature = "zstd")]
#[test]
fn zstd() {
    run_immediate_read_after_write::<ZstdVec<usize, u64>>();
}
