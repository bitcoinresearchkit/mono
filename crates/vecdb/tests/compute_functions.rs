//! Generic compute function tests for all vec types.
//!
//! These tests run against any type implementing `StoredVec`, ensuring
//! consistent compute behavior across all vec types.

use brk_exit::Exit;
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
use vecdb::{
    AnyStoredVec, EagerVec, ImportableVec, ReadableVec, Result, StoredVec, Version, WritableVec,
};

fn setup_db() -> Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

fn assert_f32_eq(actual: f32, expected: f32, tolerance: f32, message: &str) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{}: expected {}, got {} (diff: {})",
        message,
        expected,
        actual,
        (actual - expected).abs()
    );
}

fn run_compute_all_time_high<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<V> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    let values = [10, 15, 12, 20, 18, 25, 22];
    for &v in &values {
        source.push(v);
    }
    source.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_all_time_high(0, &source, &exit)?;
    result.flush()?;

    let expected = [10, 15, 15, 20, 20, 25, 25];
    for (i, v) in expected.into_iter().enumerate() {
        let actual = result.collect_one(i).unwrap();
        assert_eq!(
            actual, v,
            "All-time high mismatch at index {}: expected {}, got {}",
            i, v, actual
        );
    }

    Ok(())
}

fn run_compute_all_time_low<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<V> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    let values = [10, 5, 12, 3, 18, 2, 22];
    for &v in &values {
        source.push(v);
    }
    source.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_all_time_low(0, &source, &exit, false)?;
    result.flush()?;

    let expected = [10, 5, 5, 3, 3, 2, 2];
    for (i, v) in expected.into_iter().enumerate() {
        let actual = result.collect_one(i).unwrap();
        assert_eq!(
            actual, v,
            "All-time low mismatch at index {}: expected {}, got {}",
            i, v, actual
        );
    }

    let mut source_with_default: EagerVec<V> =
        EagerVec::forced_import(&db, "source_with_default", Version::ONE)?;
    for v in [0, 10, 5, 0, 12, 3, 0, 2] {
        source_with_default.push(v);
    }
    source_with_default.flush()?;

    let mut excluding_default: EagerVec<V> =
        EagerVec::forced_import(&db, "excluding_default", Version::ONE)?;
    excluding_default.compute_all_time_low(0, &source_with_default, &exit, true)?;
    excluding_default.flush()?;

    for (i, expected) in [0, 10, 5, 5, 5, 3, 3, 2].into_iter().enumerate() {
        assert_eq!(
            excluding_default.collect_one(i).unwrap(),
            expected,
            "Excluded default leaked into all-time low at index {i}"
        );
    }

    Ok(())
}

fn run_compute_functions_with_resume<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<V> = EagerVec::forced_import(&db, "source", Version::ONE)?;
    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;

    for i in 0..5 {
        source.push((i * 10) as u32);
    }
    source.flush()?;

    result.compute_all_time_high(0, &source, &exit)?;
    result.flush()?;

    for i in 0..5 {
        let actual = result.collect_one(i).unwrap();
        let expected = (i * 10) as u32;
        assert_eq!(actual, expected);
    }

    for i in 5..10 {
        source.push((i * 10) as u32);
    }
    source.flush()?;

    result.compute_all_time_high(0, &source, &exit)?;
    result.flush()?;

    for i in 0..10 {
        let actual = result.collect_one(i).unwrap();
        let expected = (i * 10) as u32;
        assert_eq!(actual, expected);
    }

    Ok(())
}

fn run_compute_subtract<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u64>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut vec1: EagerVec<V> = EagerVec::forced_import(&db, "vec1", Version::ONE)?;
    let mut vec2: EagerVec<V> = EagerVec::forced_import(&db, "vec2", Version::ONE)?;

    for i in 0..10 {
        vec1.push((100 + i * 10) as u64);
        vec2.push((i * 5) as u64);
    }
    vec1.flush()?;
    vec2.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_subtract(0, &vec1, &vec2, &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = (100 + i * 10 - i * 5) as u64;
        let actual = result.collect_one(i).unwrap();
        assert_eq!(actual, expected);
    }

    Ok(())
}

fn run_compute_multiply<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut vec1: EagerVec<V> = EagerVec::forced_import(&db, "vec1", Version::ONE)?;
    let mut vec2: EagerVec<V> = EagerVec::forced_import(&db, "vec2", Version::ONE)?;

    for i in 0..10 {
        vec1.push((i + 1) as u32);
        vec2.push((i + 2) as u32);
    }
    vec1.flush()?;
    vec2.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_multiply(0, &vec1, &vec2, &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = ((i + 1) * (i + 2)) as u32;
        let actual = result.collect_one(i).unwrap();
        assert_eq!(actual, expected);
    }

    Ok(())
}

fn run_compute_sma<VS, VR>() -> Result<()>
where
    VS: StoredVec<I = usize, T = u16>,
    VR: StoredVec<I = usize, T = f32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<VS> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    for i in 0..10 {
        source.push((i * 10) as u16);
    }
    source.flush()?;

    let mut result: EagerVec<VR> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_sma(0, &source, 3, &exit, None)?;
    result.flush()?;

    for i in 0..10_u64 {
        let actual = result.collect_one(i as usize).unwrap();
        if i < 2 {
            let sum: u64 = (0..=i).map(|j| j * 10).sum();
            let expected = sum as f32 / (i + 1) as f32;
            assert_f32_eq(actual, expected, 0.001, &format!("SMA at index {}", i));
        } else {
            let sum: u64 = (i - 2..=i).map(|j| j * 10).sum();
            let expected = sum as f32 / 3.0;
            assert_f32_eq(actual, expected, 0.001, &format!("SMA at index {}", i));
        }
    }

    Ok(())
}

#[test]
fn bytes() -> Result<()> {
    run_compute_all_time_high::<BytesVec<usize, u32>>()?;
    run_compute_all_time_low::<BytesVec<usize, u32>>()?;
    run_compute_functions_with_resume::<BytesVec<usize, u32>>()?;
    run_compute_subtract::<BytesVec<usize, u64>>()?;
    run_compute_multiply::<BytesVec<usize, u32>>()?;
    run_compute_sma::<BytesVec<usize, u16>, BytesVec<usize, f32>>()
}

#[cfg(feature = "zerocopy")]
#[test]
fn zerocopy() -> Result<()> {
    run_compute_all_time_high::<ZeroCopyVec<usize, u32>>()?;
    run_compute_all_time_low::<ZeroCopyVec<usize, u32>>()?;
    run_compute_functions_with_resume::<ZeroCopyVec<usize, u32>>()?;
    run_compute_subtract::<ZeroCopyVec<usize, u64>>()?;
    run_compute_multiply::<ZeroCopyVec<usize, u32>>()?;
    run_compute_sma::<ZeroCopyVec<usize, u16>, ZeroCopyVec<usize, f32>>()
}

#[cfg(feature = "pco")]
#[test]
fn pco() -> Result<()> {
    run_compute_all_time_high::<PcoVec<usize, u32>>()?;
    run_compute_all_time_low::<PcoVec<usize, u32>>()?;
    run_compute_functions_with_resume::<PcoVec<usize, u32>>()?;
    run_compute_subtract::<PcoVec<usize, u64>>()?;
    run_compute_multiply::<PcoVec<usize, u32>>()?;
    run_compute_sma::<PcoVec<usize, u16>, PcoVec<usize, f32>>()
}

#[cfg(feature = "lz4")]
#[test]
fn lz4() -> Result<()> {
    run_compute_all_time_high::<LZ4Vec<usize, u32>>()?;
    run_compute_all_time_low::<LZ4Vec<usize, u32>>()?;
    run_compute_functions_with_resume::<LZ4Vec<usize, u32>>()?;
    run_compute_subtract::<LZ4Vec<usize, u64>>()?;
    run_compute_multiply::<LZ4Vec<usize, u32>>()?;
    run_compute_sma::<LZ4Vec<usize, u16>, LZ4Vec<usize, f32>>()
}

#[cfg(feature = "zstd")]
#[test]
fn zstd() -> Result<()> {
    run_compute_all_time_high::<ZstdVec<usize, u32>>()?;
    run_compute_all_time_low::<ZstdVec<usize, u32>>()?;
    run_compute_functions_with_resume::<ZstdVec<usize, u32>>()?;
    run_compute_subtract::<ZstdVec<usize, u64>>()?;
    run_compute_multiply::<ZstdVec<usize, u32>>()?;
    run_compute_sma::<ZstdVec<usize, u16>, ZstdVec<usize, f32>>()
}
