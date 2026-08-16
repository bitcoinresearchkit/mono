//! Generic compute function tests for all vec types.
//!
//! These tests run against any type implementing `StoredVec`, ensuring
//! consistent compute behavior across all vec types.

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, EagerVec, Exit, ImportableVec, ReadableVec, Result, StoredVec, Version,
    WritableVec,
};

// ============================================================================
// Test Setup
// ============================================================================

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

// ============================================================================
// Generic Test Functions
// ============================================================================

fn run_compute_sum_of_others<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u64>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut vec1: EagerVec<V> = EagerVec::forced_import(&db, "vec1", Version::ONE)?;
    let mut vec2: EagerVec<V> = EagerVec::forced_import(&db, "vec2", Version::ONE)?;
    let mut vec3: EagerVec<V> = EagerVec::forced_import(&db, "vec3", Version::ONE)?;

    for i in 0..10 {
        vec1.push((i * 10) as u64);
        vec2.push((i * 5) as u64);
        vec3.push(i as u64);
    }
    vec1.flush()?;
    vec2.flush()?;
    vec3.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_sum_of_others(0, &[&vec1, &vec2, &vec3], &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = ((i * 10) + (i * 5) + i) as u64;
        let actual = result.collect_one(i).unwrap();
        assert_eq!(
            actual, expected,
            "Sum mismatch at index {}: expected {}, got {}",
            i, expected, actual
        );
    }

    Ok(())
}

fn run_compute_min_of_others<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u64>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut vec1: EagerVec<V> = EagerVec::forced_import(&db, "vec1", Version::ONE)?;
    let mut vec2: EagerVec<V> = EagerVec::forced_import(&db, "vec2", Version::ONE)?;
    let mut vec3: EagerVec<V> = EagerVec::forced_import(&db, "vec3", Version::ONE)?;

    for i in 0..10 {
        vec1.push((50 + i) as u64);
        vec2.push((10 + i) as u64);
        vec3.push((100 + i) as u64);
    }
    vec1.flush()?;
    vec2.flush()?;
    vec3.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_min_of_others(0, &[&vec1, &vec2, &vec3], &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = (10 + i) as u64;
        let actual = result.collect_one(i).unwrap();
        assert_eq!(
            actual, expected,
            "Min mismatch at index {}: expected {}, got {}",
            i, expected, actual
        );
    }

    Ok(())
}

fn run_compute_max_of_others<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u64>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut vec1: EagerVec<V> = EagerVec::forced_import(&db, "vec1", Version::ONE)?;
    let mut vec2: EagerVec<V> = EagerVec::forced_import(&db, "vec2", Version::ONE)?;
    let mut vec3: EagerVec<V> = EagerVec::forced_import(&db, "vec3", Version::ONE)?;

    for i in 0..10 {
        vec1.push((50 + i) as u64);
        vec2.push((10 + i) as u64);
        vec3.push((100 + i) as u64);
    }
    vec1.flush()?;
    vec2.flush()?;
    vec3.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_max_of_others(0, &[&vec1, &vec2, &vec3], &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = (100 + i) as u64;
        let actual = result.collect_one(i).unwrap();
        assert_eq!(
            actual, expected,
            "Max mismatch at index {}: expected {}, got {}",
            i, expected, actual
        );
    }

    Ok(())
}

fn run_compute_previous_value<VS, VR>() -> Result<()>
where
    VS: StoredVec<I = usize, T = u16>,
    VR: StoredVec<I = usize, T = f32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source = EagerVec::<VS>::forced_import(&db, "source", Version::ONE)?;

    for i in 0..5 {
        source.push(((i + 1) * 10) as u16);
    }
    source.flush()?;

    let mut result = EagerVec::<VR>::forced_import(&db, "result", Version::ONE)?;
    result.compute_previous_value(0, &source, 1, &exit)?;
    result.flush()?;

    let actual_0 = result.collect_first().unwrap();
    assert!(
        actual_0.is_nan(),
        "First element should be NaN when no previous value exists"
    );

    let expected = [10.0, 20.0, 30.0, 40.0];
    for (i, v) in expected.into_iter().enumerate() {
        let actual = result.collect_one(i + 1).unwrap();
        assert_eq!(
            actual,
            v,
            "Previous value mismatch at index {}: expected {}, got {}",
            i + 1,
            v,
            actual
        );
    }

    Ok(())
}

fn run_compute_change<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<V> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    let values = [10, 20, 25, 30, 50];
    for &v in &values {
        source.push(v);
    }
    source.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_change(0, &source, 1, &exit)?;
    result.flush()?;

    let expected = [0, 10, 5, 5, 20];
    for (i, v) in expected.into_iter().enumerate() {
        let actual = result.collect_one(i).unwrap();
        assert_eq!(
            actual, v,
            "Change mismatch at index {}: expected {}, got {}",
            i, v, actual
        );
    }

    Ok(())
}

fn run_compute_percentage_change<VS, VR>() -> Result<()>
where
    VS: StoredVec<I = usize, T = u16>,
    VR: StoredVec<I = usize, T = f32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<VS> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    let values = [100, 110, 121, 133];
    for &v in &values {
        source.push(v);
    }
    source.flush()?;

    let mut result: EagerVec<VR> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_percentage_change(0, &source, 1, &exit)?;
    result.flush()?;

    let actual_0 = result.collect_first().unwrap();
    let actual_1 = result.collect_one(1).unwrap();
    let actual_2 = result.collect_one(2).unwrap();
    let actual_3 = result.collect_one(3).unwrap();

    assert!(
        actual_0.is_nan(),
        "First element should be NaN when no previous value exists"
    );
    assert!((actual_1 - 10.0).abs() < 0.01);
    assert!((actual_2 - 10.0).abs() < 0.01);
    assert!((actual_3 - 9.917).abs() < 0.01);

    Ok(())
}

fn run_compute_sliding_window_max<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<V> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    let values = [3, 1, 4, 1, 5, 9, 2, 6];
    for &v in &values {
        source.push(v);
    }
    source.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_max(0, &source, 3, &exit)?;
    result.flush()?;

    let expected = [3, 3, 4, 4, 5, 9, 9, 9];
    for (i, v) in expected.into_iter().enumerate() {
        let actual = result.collect_one(i).unwrap();
        assert_eq!(
            actual, v,
            "Sliding window max mismatch at index {}: expected {}, got {}",
            i, v, actual
        );
    }

    Ok(())
}

fn run_compute_sliding_window_min<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<V> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    let values = [3, 1, 4, 1, 5, 9, 2, 6];
    for &v in &values {
        source.push(v);
    }
    source.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_min(0, &source, 3, &exit)?;
    result.flush()?;

    let expected = [3, 1, 1, 1, 1, 1, 2, 2];
    for (i, v) in expected.into_iter().enumerate() {
        let actual = result.collect_one(i).unwrap();
        assert_eq!(
            actual, v,
            "Sliding window min mismatch at index {}: expected {}, got {}",
            i, v, actual
        );
    }

    Ok(())
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
    result.compute_all_time_low_(0, &source, &exit, false)?;
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
    excluding_default.compute_all_time_low_(0, &source_with_default, &exit, true)?;
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

fn run_compute_cagr<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = f32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut percentage_returns: EagerVec<V> =
        EagerVec::forced_import(&db, "returns", Version::ONE)?;

    for _ in 0..5 {
        percentage_returns.push(100.0);
    }
    percentage_returns.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_cagr(0, &percentage_returns, 730, &exit)?;
    result.flush()?;

    for i in 0..5 {
        let actual = result.collect_one(i).unwrap();
        let expected = 41.42;
        assert!(
            (actual - expected).abs() < 0.01,
            "CAGR mismatch at index {}: expected {}, got {}",
            i,
            expected,
            actual
        );
    }

    Ok(())
}

fn run_compute_zscore<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = f32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<V> = EagerVec::forced_import(&db, "source", Version::ONE)?;
    let mut sma: EagerVec<V> = EagerVec::forced_import(&db, "sma", Version::ONE)?;
    let mut sd: EagerVec<V> = EagerVec::forced_import(&db, "sd", Version::ONE)?;

    for i in 0..4 {
        source.push(10.0 + i as f32 * 2.0);
        sma.push(10.0);
        sd.push(2.0);
    }
    source.flush()?;
    sma.flush()?;
    sd.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_zscore(0, &source, &sma, &sd, &exit)?;
    result.flush()?;

    let expected = [0.0, 1.0, 2.0, 3.0];
    for (i, v) in expected.into_iter().enumerate() {
        let actual = result.collect_one(i).unwrap();
        assert!(
            (actual - v).abs() < 0.01,
            "Z-score mismatch at index {}: expected {}, got {}",
            i,
            v,
            actual
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

fn run_compute_add<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u64>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut vec1: EagerVec<V> = EagerVec::forced_import(&db, "vec1", Version::ONE)?;
    let mut vec2: EagerVec<V> = EagerVec::forced_import(&db, "vec2", Version::ONE)?;

    for i in 0..10 {
        vec1.push((i * 10) as u64);
        vec2.push((i * 5) as u64);
    }
    vec1.flush()?;
    vec2.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_add(0, &vec1, &vec2, &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = (i * 10 + i * 5) as u64;
        let actual = result.collect_one(i).unwrap();
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

fn run_compute_divide<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = f32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut vec1: EagerVec<V> = EagerVec::forced_import(&db, "vec1", Version::ONE)?;
    let mut vec2: EagerVec<V> = EagerVec::forced_import(&db, "vec2", Version::ONE)?;

    for i in 0..10 {
        vec1.push(100.0 + i as f32 * 10.0);
        vec2.push(i as f32 + 1.0);
    }
    vec1.flush()?;
    vec2.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_divide(0, &vec1, &vec2, &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = (100.0 + i as f32 * 10.0) / (i as f32 + 1.0);
        let actual = result.collect_one(i).unwrap();
        assert_f32_eq(actual, expected, 0.001, &format!("Divide at index {}", i));
    }

    Ok(())
}

fn run_compute_max<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u64>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<V> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    for i in 0..10 {
        let value = if i < 5 { i * 10 } else { (9 - i) * 10 };
        source.push(value as u64);
    }
    source.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_max(0, &source, usize::MAX, &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = if i < 5 { (i * 10) as u64 } else { 40u64 };
        let actual = result.collect_one(i).unwrap();
        assert_eq!(actual, expected, "Max at index {}", i);
    }

    Ok(())
}

fn run_compute_min<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u64>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<V> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    for i in 0..10 {
        let value = if i < 5 {
            100 - i * 10
        } else {
            50 + (i - 5) * 10
        };
        source.push(value as u64);
    }
    source.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_min(0, &source, usize::MAX, &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = if i < 5 { (100 - i * 10) as u64 } else { 50u64 };
        let actual = result.collect_one(i).unwrap();
        assert_eq!(actual, expected, "Min at index {}", i);
    }

    Ok(())
}

fn run_compute_sum<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u64>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<V> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    for i in 0..10 {
        source.push((i + 1) as u64);
    }
    source.flush()?;

    let mut result: EagerVec<V> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_sum(0, &source, usize::MAX, &exit)?;
    result.flush()?;

    let mut expected_sum = 0u64;
    for i in 0..10 {
        expected_sum += (i + 1) as u64;
        let actual = result.collect_one(i).unwrap();
        assert_eq!(actual, expected_sum, "Cumulative sum at index {}", i);
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
    result.compute_sma(0, &source, 3, &exit)?;
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

fn run_compute_ema<VS, VR>() -> Result<()>
where
    VS: StoredVec<I = usize, T = u16>,
    VR: StoredVec<I = usize, T = f32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut source: EagerVec<VS> = EagerVec::forced_import(&db, "source", Version::ONE)?;

    for _ in 0..10 {
        source.push(100);
    }
    source.flush()?;

    let mut result: EagerVec<VR> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_ema(0, &source, 3, &exit)?;
    result.flush()?;

    for i in 0..10 {
        let actual = result.collect_one(i).unwrap();
        assert_f32_eq(actual, 100.0, 0.1, &format!("EMA at index {}", i));
    }

    Ok(())
}

fn run_compute_percentage<VS, VR>() -> Result<()>
where
    VS: StoredVec<I = usize, T = u16>,
    VR: StoredVec<I = usize, T = f32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut numerator: EagerVec<VS> = EagerVec::forced_import(&db, "numerator", Version::ONE)?;
    let mut denominator: EagerVec<VS> = EagerVec::forced_import(&db, "denominator", Version::ONE)?;

    for i in 0..10 {
        numerator.push((i + 1) as u16);
        denominator.push(10);
    }
    numerator.flush()?;
    denominator.flush()?;

    let mut result: EagerVec<VR> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_percentage(0, &numerator, &denominator, &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = ((i + 1) as f32 / 10.0) * 100.0;
        let actual = result.collect_one(i).unwrap();
        assert_f32_eq(
            actual,
            expected,
            0.001,
            &format!("Percentage at index {}", i),
        );
    }

    Ok(())
}

fn run_compute_percentage_difference<VS, VR>() -> Result<()>
where
    VS: StoredVec<I = usize, T = u16>,
    VR: StoredVec<I = usize, T = f32>,
{
    let (db, _temp) = setup_db()?;
    let exit = Exit::new();

    let mut vec1: EagerVec<VS> = EagerVec::forced_import(&db, "vec1", Version::ONE)?;
    let mut vec2: EagerVec<VS> = EagerVec::forced_import(&db, "vec2", Version::ONE)?;

    for i in 0..10 {
        vec1.push((100 + i * 10) as u16);
        vec2.push(100);
    }
    vec1.flush()?;
    vec2.flush()?;

    let mut result: EagerVec<VR> = EagerVec::forced_import(&db, "result", Version::ONE)?;
    result.compute_percentage_difference(0, &vec1, &vec2, &exit)?;
    result.flush()?;

    for i in 0..10 {
        let expected = (((100 + i * 10) as f32 - 100.0) / 100.0) * 100.0;
        let actual = result.collect_one(i).unwrap();
        assert_f32_eq(
            actual,
            expected,
            0.001,
            &format!("Percentage difference at index {}", i),
        );
    }

    Ok(())
}

// ============================================================================
// Test instantiation for BytesVec (no feature flag needed)
// ============================================================================

mod bytes {
    use super::*;
    use vecdb::BytesVec;

    #[test]
    fn compute_sum_of_others() -> Result<()> {
        run_compute_sum_of_others::<BytesVec<usize, u64>>()
    }

    #[test]
    fn compute_min_of_others() -> Result<()> {
        run_compute_min_of_others::<BytesVec<usize, u64>>()
    }

    #[test]
    fn compute_max_of_others() -> Result<()> {
        run_compute_max_of_others::<BytesVec<usize, u64>>()
    }

    #[test]
    fn compute_previous_value() -> Result<()> {
        run_compute_previous_value::<BytesVec<usize, u16>, BytesVec<usize, f32>>()
    }

    #[test]
    fn compute_change() -> Result<()> {
        run_compute_change::<BytesVec<usize, u32>>()
    }

    #[test]
    fn compute_percentage_change() -> Result<()> {
        run_compute_percentage_change::<BytesVec<usize, u16>, BytesVec<usize, f32>>()
    }

    #[test]
    fn compute_sliding_window_max() -> Result<()> {
        run_compute_sliding_window_max::<BytesVec<usize, u32>>()
    }

    #[test]
    fn compute_sliding_window_min() -> Result<()> {
        run_compute_sliding_window_min::<BytesVec<usize, u32>>()
    }

    #[test]
    fn compute_all_time_high() -> Result<()> {
        run_compute_all_time_high::<BytesVec<usize, u32>>()
    }

    #[test]
    fn compute_all_time_low() -> Result<()> {
        run_compute_all_time_low::<BytesVec<usize, u32>>()
    }

    #[test]
    fn compute_cagr() -> Result<()> {
        run_compute_cagr::<BytesVec<usize, f32>>()
    }

    #[test]
    fn compute_zscore() -> Result<()> {
        run_compute_zscore::<BytesVec<usize, f32>>()
    }

    #[test]
    fn compute_functions_with_resume() -> Result<()> {
        run_compute_functions_with_resume::<BytesVec<usize, u32>>()
    }

    #[test]
    fn compute_add() -> Result<()> {
        run_compute_add::<BytesVec<usize, u64>>()
    }

    #[test]
    fn compute_subtract() -> Result<()> {
        run_compute_subtract::<BytesVec<usize, u64>>()
    }

    #[test]
    fn compute_multiply() -> Result<()> {
        run_compute_multiply::<BytesVec<usize, u32>>()
    }

    #[test]
    fn compute_divide() -> Result<()> {
        run_compute_divide::<BytesVec<usize, f32>>()
    }

    #[test]
    fn compute_max() -> Result<()> {
        run_compute_max::<BytesVec<usize, u64>>()
    }

    #[test]
    fn compute_min() -> Result<()> {
        run_compute_min::<BytesVec<usize, u64>>()
    }

    #[test]
    fn compute_sum() -> Result<()> {
        run_compute_sum::<BytesVec<usize, u64>>()
    }

    #[test]
    fn compute_sma() -> Result<()> {
        run_compute_sma::<BytesVec<usize, u16>, BytesVec<usize, f32>>()
    }

    #[test]
    fn compute_ema() -> Result<()> {
        run_compute_ema::<BytesVec<usize, u16>, BytesVec<usize, f32>>()
    }

    #[test]
    fn compute_percentage() -> Result<()> {
        run_compute_percentage::<BytesVec<usize, u16>, BytesVec<usize, f32>>()
    }

    #[test]
    fn compute_percentage_difference() -> Result<()> {
        run_compute_percentage_difference::<BytesVec<usize, u16>, BytesVec<usize, f32>>()
    }
}

// ============================================================================
// Test instantiation for feature-gated vec types
// ============================================================================

#[cfg(feature = "zerocopy")]
mod zerocopy {
    use super::*;
    use vecdb::ZeroCopyVec;

    #[test]
    fn compute_sum_of_others() -> Result<()> {
        run_compute_sum_of_others::<ZeroCopyVec<usize, u64>>()
    }

    #[test]
    fn compute_min_of_others() -> Result<()> {
        run_compute_min_of_others::<ZeroCopyVec<usize, u64>>()
    }

    #[test]
    fn compute_max_of_others() -> Result<()> {
        run_compute_max_of_others::<ZeroCopyVec<usize, u64>>()
    }

    #[test]
    fn compute_previous_value() -> Result<()> {
        run_compute_previous_value::<ZeroCopyVec<usize, u16>, ZeroCopyVec<usize, f32>>()
    }

    #[test]
    fn compute_change() -> Result<()> {
        run_compute_change::<ZeroCopyVec<usize, u32>>()
    }

    #[test]
    fn compute_percentage_change() -> Result<()> {
        run_compute_percentage_change::<ZeroCopyVec<usize, u16>, ZeroCopyVec<usize, f32>>()
    }

    #[test]
    fn compute_sliding_window_max() -> Result<()> {
        run_compute_sliding_window_max::<ZeroCopyVec<usize, u32>>()
    }

    #[test]
    fn compute_sliding_window_min() -> Result<()> {
        run_compute_sliding_window_min::<ZeroCopyVec<usize, u32>>()
    }

    #[test]
    fn compute_all_time_high() -> Result<()> {
        run_compute_all_time_high::<ZeroCopyVec<usize, u32>>()
    }

    #[test]
    fn compute_all_time_low() -> Result<()> {
        run_compute_all_time_low::<ZeroCopyVec<usize, u32>>()
    }

    #[test]
    fn compute_cagr() -> Result<()> {
        run_compute_cagr::<ZeroCopyVec<usize, f32>>()
    }

    #[test]
    fn compute_zscore() -> Result<()> {
        run_compute_zscore::<ZeroCopyVec<usize, f32>>()
    }

    #[test]
    fn compute_functions_with_resume() -> Result<()> {
        run_compute_functions_with_resume::<ZeroCopyVec<usize, u32>>()
    }

    #[test]
    fn compute_add() -> Result<()> {
        run_compute_add::<ZeroCopyVec<usize, u64>>()
    }

    #[test]
    fn compute_subtract() -> Result<()> {
        run_compute_subtract::<ZeroCopyVec<usize, u64>>()
    }

    #[test]
    fn compute_multiply() -> Result<()> {
        run_compute_multiply::<ZeroCopyVec<usize, u32>>()
    }

    #[test]
    fn compute_divide() -> Result<()> {
        run_compute_divide::<ZeroCopyVec<usize, f32>>()
    }

    #[test]
    fn compute_max() -> Result<()> {
        run_compute_max::<ZeroCopyVec<usize, u64>>()
    }

    #[test]
    fn compute_min() -> Result<()> {
        run_compute_min::<ZeroCopyVec<usize, u64>>()
    }

    #[test]
    fn compute_sum() -> Result<()> {
        run_compute_sum::<ZeroCopyVec<usize, u64>>()
    }

    #[test]
    fn compute_sma() -> Result<()> {
        run_compute_sma::<ZeroCopyVec<usize, u16>, ZeroCopyVec<usize, f32>>()
    }

    #[test]
    fn compute_ema() -> Result<()> {
        run_compute_ema::<ZeroCopyVec<usize, u16>, ZeroCopyVec<usize, f32>>()
    }

    #[test]
    fn compute_percentage() -> Result<()> {
        run_compute_percentage::<ZeroCopyVec<usize, u16>, ZeroCopyVec<usize, f32>>()
    }

    #[test]
    fn compute_percentage_difference() -> Result<()> {
        run_compute_percentage_difference::<ZeroCopyVec<usize, u16>, ZeroCopyVec<usize, f32>>()
    }
}

#[cfg(feature = "pco")]
mod pco {
    use super::*;
    use vecdb::PcoVec;

    #[test]
    fn compute_sum_of_others() -> Result<()> {
        run_compute_sum_of_others::<PcoVec<usize, u64>>()
    }

    #[test]
    fn compute_min_of_others() -> Result<()> {
        run_compute_min_of_others::<PcoVec<usize, u64>>()
    }

    #[test]
    fn compute_max_of_others() -> Result<()> {
        run_compute_max_of_others::<PcoVec<usize, u64>>()
    }

    #[test]
    fn compute_previous_value() -> Result<()> {
        run_compute_previous_value::<PcoVec<usize, u16>, PcoVec<usize, f32>>()
    }

    #[test]
    fn compute_change() -> Result<()> {
        run_compute_change::<PcoVec<usize, u32>>()
    }

    #[test]
    fn compute_percentage_change() -> Result<()> {
        run_compute_percentage_change::<PcoVec<usize, u16>, PcoVec<usize, f32>>()
    }

    #[test]
    fn compute_sliding_window_max() -> Result<()> {
        run_compute_sliding_window_max::<PcoVec<usize, u32>>()
    }

    #[test]
    fn compute_sliding_window_min() -> Result<()> {
        run_compute_sliding_window_min::<PcoVec<usize, u32>>()
    }

    #[test]
    fn compute_all_time_high() -> Result<()> {
        run_compute_all_time_high::<PcoVec<usize, u32>>()
    }

    #[test]
    fn compute_all_time_low() -> Result<()> {
        run_compute_all_time_low::<PcoVec<usize, u32>>()
    }

    #[test]
    fn compute_cagr() -> Result<()> {
        run_compute_cagr::<PcoVec<usize, f32>>()
    }

    #[test]
    fn compute_zscore() -> Result<()> {
        run_compute_zscore::<PcoVec<usize, f32>>()
    }

    #[test]
    fn compute_functions_with_resume() -> Result<()> {
        run_compute_functions_with_resume::<PcoVec<usize, u32>>()
    }

    #[test]
    fn compute_add() -> Result<()> {
        run_compute_add::<PcoVec<usize, u64>>()
    }

    #[test]
    fn compute_subtract() -> Result<()> {
        run_compute_subtract::<PcoVec<usize, u64>>()
    }

    #[test]
    fn compute_multiply() -> Result<()> {
        run_compute_multiply::<PcoVec<usize, u32>>()
    }

    #[test]
    fn compute_divide() -> Result<()> {
        run_compute_divide::<PcoVec<usize, f32>>()
    }

    #[test]
    fn compute_max() -> Result<()> {
        run_compute_max::<PcoVec<usize, u64>>()
    }

    #[test]
    fn compute_min() -> Result<()> {
        run_compute_min::<PcoVec<usize, u64>>()
    }

    #[test]
    fn compute_sum() -> Result<()> {
        run_compute_sum::<PcoVec<usize, u64>>()
    }

    #[test]
    fn compute_sma() -> Result<()> {
        run_compute_sma::<PcoVec<usize, u16>, PcoVec<usize, f32>>()
    }

    #[test]
    fn compute_ema() -> Result<()> {
        run_compute_ema::<PcoVec<usize, u16>, PcoVec<usize, f32>>()
    }

    #[test]
    fn compute_percentage() -> Result<()> {
        run_compute_percentage::<PcoVec<usize, u16>, PcoVec<usize, f32>>()
    }

    #[test]
    fn compute_percentage_difference() -> Result<()> {
        run_compute_percentage_difference::<PcoVec<usize, u16>, PcoVec<usize, f32>>()
    }
}

#[cfg(feature = "lz4")]
mod lz4 {
    use super::*;
    use vecdb::LZ4Vec;

    #[test]
    fn compute_sum_of_others() -> Result<()> {
        run_compute_sum_of_others::<LZ4Vec<usize, u64>>()
    }

    #[test]
    fn compute_min_of_others() -> Result<()> {
        run_compute_min_of_others::<LZ4Vec<usize, u64>>()
    }

    #[test]
    fn compute_max_of_others() -> Result<()> {
        run_compute_max_of_others::<LZ4Vec<usize, u64>>()
    }

    #[test]
    fn compute_previous_value() -> Result<()> {
        run_compute_previous_value::<LZ4Vec<usize, u16>, LZ4Vec<usize, f32>>()
    }

    #[test]
    fn compute_change() -> Result<()> {
        run_compute_change::<LZ4Vec<usize, u32>>()
    }

    #[test]
    fn compute_percentage_change() -> Result<()> {
        run_compute_percentage_change::<LZ4Vec<usize, u16>, LZ4Vec<usize, f32>>()
    }

    #[test]
    fn compute_sliding_window_max() -> Result<()> {
        run_compute_sliding_window_max::<LZ4Vec<usize, u32>>()
    }

    #[test]
    fn compute_sliding_window_min() -> Result<()> {
        run_compute_sliding_window_min::<LZ4Vec<usize, u32>>()
    }

    #[test]
    fn compute_all_time_high() -> Result<()> {
        run_compute_all_time_high::<LZ4Vec<usize, u32>>()
    }

    #[test]
    fn compute_all_time_low() -> Result<()> {
        run_compute_all_time_low::<LZ4Vec<usize, u32>>()
    }

    #[test]
    fn compute_cagr() -> Result<()> {
        run_compute_cagr::<LZ4Vec<usize, f32>>()
    }

    #[test]
    fn compute_zscore() -> Result<()> {
        run_compute_zscore::<LZ4Vec<usize, f32>>()
    }

    #[test]
    fn compute_functions_with_resume() -> Result<()> {
        run_compute_functions_with_resume::<LZ4Vec<usize, u32>>()
    }

    #[test]
    fn compute_add() -> Result<()> {
        run_compute_add::<LZ4Vec<usize, u64>>()
    }

    #[test]
    fn compute_subtract() -> Result<()> {
        run_compute_subtract::<LZ4Vec<usize, u64>>()
    }

    #[test]
    fn compute_multiply() -> Result<()> {
        run_compute_multiply::<LZ4Vec<usize, u32>>()
    }

    #[test]
    fn compute_divide() -> Result<()> {
        run_compute_divide::<LZ4Vec<usize, f32>>()
    }

    #[test]
    fn compute_max() -> Result<()> {
        run_compute_max::<LZ4Vec<usize, u64>>()
    }

    #[test]
    fn compute_min() -> Result<()> {
        run_compute_min::<LZ4Vec<usize, u64>>()
    }

    #[test]
    fn compute_sum() -> Result<()> {
        run_compute_sum::<LZ4Vec<usize, u64>>()
    }

    #[test]
    fn compute_sma() -> Result<()> {
        run_compute_sma::<LZ4Vec<usize, u16>, LZ4Vec<usize, f32>>()
    }

    #[test]
    fn compute_ema() -> Result<()> {
        run_compute_ema::<LZ4Vec<usize, u16>, LZ4Vec<usize, f32>>()
    }

    #[test]
    fn compute_percentage() -> Result<()> {
        run_compute_percentage::<LZ4Vec<usize, u16>, LZ4Vec<usize, f32>>()
    }

    #[test]
    fn compute_percentage_difference() -> Result<()> {
        run_compute_percentage_difference::<LZ4Vec<usize, u16>, LZ4Vec<usize, f32>>()
    }
}

#[cfg(feature = "zstd")]
mod zstd {
    use super::*;
    use vecdb::ZstdVec;

    #[test]
    fn compute_sum_of_others() -> Result<()> {
        run_compute_sum_of_others::<ZstdVec<usize, u64>>()
    }

    #[test]
    fn compute_min_of_others() -> Result<()> {
        run_compute_min_of_others::<ZstdVec<usize, u64>>()
    }

    #[test]
    fn compute_max_of_others() -> Result<()> {
        run_compute_max_of_others::<ZstdVec<usize, u64>>()
    }

    #[test]
    fn compute_previous_value() -> Result<()> {
        run_compute_previous_value::<ZstdVec<usize, u16>, ZstdVec<usize, f32>>()
    }

    #[test]
    fn compute_change() -> Result<()> {
        run_compute_change::<ZstdVec<usize, u32>>()
    }

    #[test]
    fn compute_percentage_change() -> Result<()> {
        run_compute_percentage_change::<ZstdVec<usize, u16>, ZstdVec<usize, f32>>()
    }

    #[test]
    fn compute_sliding_window_max() -> Result<()> {
        run_compute_sliding_window_max::<ZstdVec<usize, u32>>()
    }

    #[test]
    fn compute_sliding_window_min() -> Result<()> {
        run_compute_sliding_window_min::<ZstdVec<usize, u32>>()
    }

    #[test]
    fn compute_all_time_high() -> Result<()> {
        run_compute_all_time_high::<ZstdVec<usize, u32>>()
    }

    #[test]
    fn compute_all_time_low() -> Result<()> {
        run_compute_all_time_low::<ZstdVec<usize, u32>>()
    }

    #[test]
    fn compute_cagr() -> Result<()> {
        run_compute_cagr::<ZstdVec<usize, f32>>()
    }

    #[test]
    fn compute_zscore() -> Result<()> {
        run_compute_zscore::<ZstdVec<usize, f32>>()
    }

    #[test]
    fn compute_functions_with_resume() -> Result<()> {
        run_compute_functions_with_resume::<ZstdVec<usize, u32>>()
    }

    #[test]
    fn compute_add() -> Result<()> {
        run_compute_add::<ZstdVec<usize, u64>>()
    }

    #[test]
    fn compute_subtract() -> Result<()> {
        run_compute_subtract::<ZstdVec<usize, u64>>()
    }

    #[test]
    fn compute_multiply() -> Result<()> {
        run_compute_multiply::<ZstdVec<usize, u32>>()
    }

    #[test]
    fn compute_divide() -> Result<()> {
        run_compute_divide::<ZstdVec<usize, f32>>()
    }

    #[test]
    fn compute_max() -> Result<()> {
        run_compute_max::<ZstdVec<usize, u64>>()
    }

    #[test]
    fn compute_min() -> Result<()> {
        run_compute_min::<ZstdVec<usize, u64>>()
    }

    #[test]
    fn compute_sum() -> Result<()> {
        run_compute_sum::<ZstdVec<usize, u64>>()
    }

    #[test]
    fn compute_sma() -> Result<()> {
        run_compute_sma::<ZstdVec<usize, u16>, ZstdVec<usize, f32>>()
    }

    #[test]
    fn compute_ema() -> Result<()> {
        run_compute_ema::<ZstdVec<usize, u16>, ZstdVec<usize, f32>>()
    }

    #[test]
    fn compute_percentage() -> Result<()> {
        run_compute_percentage::<ZstdVec<usize, u16>, ZstdVec<usize, f32>>()
    }

    #[test]
    fn compute_percentage_difference() -> Result<()> {
        run_compute_percentage_difference::<ZstdVec<usize, u16>, ZstdVec<usize, f32>>()
    }
}
