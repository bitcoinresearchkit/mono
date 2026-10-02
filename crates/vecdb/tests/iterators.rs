//! Generic iterator tests for all vec types.
//!
//! These tests run against any type implementing `StoredVec`, ensuring
//! consistent iterator behavior across BytesVec, ZeroCopyVec, PcoVec, LZ4Vec, and ZstdVec.

use vecdb::BytesVec;
#[cfg(any(feature = "pco", feature = "zerocopy"))]
use vecdb::EagerVec;
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
use vecdb::{ReadableVec, Result, StoredVec, Version};

fn setup_db() -> Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

fn run_ranges<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = i32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;
    assert!(vec.collect().is_empty());
    assert_eq!(vec.collect_first(), None);
    assert_eq!(vec.collect_last(), None);

    for i in 0..10_000 {
        vec.push(i);
    }
    vec.write()?;
    let expected: Vec<i32> = (0..10_000).collect();
    assert_eq!(vec.collect(), expected);
    assert_eq!(vec.collect_first(), Some(0));
    assert_eq!(vec.collect_last(), Some(9_999));
    for i in [100, 500, 50] {
        assert_eq!(vec.collect_one(i), Some(i as i32));
    }
    for (from, to) in [
        (0, 0),
        (0, 25),
        (50, 52),
        (150, 250),
        (1, 10_000),
        (9_999, 10_000),
        (10_000, 10_000),
    ] {
        assert_eq!(vec.collect_range(from, to), expected[from..to]);
    }
    assert_eq!(vec.collect_signed_range(Some(-5), None), expected[9_995..]);
    assert_eq!(vec.collect_signed_range(Some(5), Some(10)), expected[5..10]);
    Ok(())
}

fn run_stored_and_pushed<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = i32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = V::forced_import(&db, "test", Version::ONE)?;
    for (stored, pushed) in [(50, 50), (8_000, 4_000), (10_000, 100)] {
        vec.reset()?;
        for i in 0..stored {
            vec.push(i);
        }
        let prefix: Vec<i32> = (0..stored).collect();
        assert_eq!(vec.collect(), prefix);
        vec.write()?;
        assert_eq!(vec.collect(), prefix);
        assert_eq!(vec.collect_last(), Some(stored - 1));
        let end = stored + pushed;
        for i in stored..end {
            vec.push(i);
        }
        let expected: Vec<i32> = (0..end).collect();
        assert_eq!(vec.len(), end as usize);
        assert_eq!(vec.collect(), expected);
        assert_eq!(vec.collect_last(), Some(end - 1));
        for (from, to) in [
            (0, end),
            (1, end),
            (stored - 1, stored + 1),
            (stored - 10, stored + 10),
            (stored, end),
            (end, end),
        ] {
            assert_eq!(
                vec.collect_range(from as usize, to as usize),
                expected[from as usize..to as usize]
            );
        }
        let indices = [0, 2, 2, stored - 1, stored, end - 1, end].map(|i| i as usize);
        let mut sorted = vec![-1];
        vec.read_sorted_into_at(&indices, &mut sorted);
        assert_eq!(sorted, [-1, 0, 2, 2, stored - 1, stored, end - 1]);
        vec.flush()?;
        assert_eq!(vec.collect(), expected);
        for i in [stored - 1, stored, end - 1] {
            assert_eq!(vec.collect_one(i as usize), Some(i));
        }
        assert_eq!(vec.read_only_clone().read_sorted_at(&indices), sorted[1..]);
    }
    Ok(())
}

fn run<V: StoredVec<I = usize, T = i32>>() -> Result<()> {
    run_ranges::<V>()?;
    run_stored_and_pushed::<V>()
}

#[test]
fn bytes() -> Result<()> {
    run::<BytesVec<usize, i32>>()
}

#[cfg(feature = "zerocopy")]
#[test]
fn zerocopy() -> Result<()> {
    run::<ZeroCopyVec<usize, i32>>()
}

#[cfg(feature = "pco")]
#[test]
fn pco() -> Result<()> {
    run::<PcoVec<usize, i32>>()
}

#[cfg(feature = "lz4")]
#[test]
fn lz4() -> Result<()> {
    run::<LZ4Vec<usize, i32>>()
}

#[cfg(feature = "zstd")]
#[test]
fn zstd() -> Result<()> {
    run::<ZstdVec<usize, i32>>()
}

#[cfg(feature = "zerocopy")]
#[test]
fn eager_zerocopy() -> Result<()> {
    run::<EagerVec<ZeroCopyVec<usize, i32>>>()
}

#[cfg(feature = "pco")]
#[test]
fn eager_pco() -> Result<()> {
    run::<EagerVec<PcoVec<usize, i32>>>()
}
