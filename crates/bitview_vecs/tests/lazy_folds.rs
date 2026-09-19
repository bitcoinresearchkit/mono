use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use bitview_transforms::RatioU64;
use bitview_vecs::{
    LazyIndexCountVec, LazyIndexedVec, LazyLookbackVec, LazyPreviousDeltaVec, LazyRollingRatioVec,
    LazySinceDayVec, LazyWindowVec,
};
use brk_types::{Day1, Height, PartsPerMillion32, StoredU64, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BinaryTransform, Database, EagerVec, ImportableVec, PcoVec, PcoVecValue,
    ReadBounds, ReadableVec, ReverseOperands, VecValue, WritableVec,
};

#[allow(dead_code)]
mod common;

fn stored<T: PcoVecValue>(
    db: &Database,
    name: &str,
    values: impl IntoIterator<Item = T>,
) -> EagerVec<PcoVec<Height, T>> {
    let mut source = EagerVec::forced_import(db, name, Version::ONE).unwrap();
    for value in values {
        source.push(value);
    }
    source.write().unwrap();
    source
}

fn check_folds<T: VecValue + PartialEq>(source: &impl ReadableVec<Height, T>) {
    for (from, to) in [
        (0, 0),
        (0, 1),
        (0, 64),
        (1, 8),
        (20, 24),
        (40, 80),
        (64, 64),
        (80, 64),
        (usize::MAX, usize::MAX),
    ] {
        let expected = source.collect_range_at(from.min(80), to.min(80));
        assert_eq!(
            source.fold_range_at(from, to, Vec::new(), |mut values, value| {
                values.push(value);
                values
            }),
            expected,
        );
        assert_eq!(
            source.try_fold_range_at(from, to, Vec::new(), |mut values, value| {
                values.push(value);
                Ok::<_, ()>(values)
            }),
            Ok(expected.clone()),
        );
        for fail_at in 0..expected.len() {
            let mut visited = Vec::new();
            let result = source.try_fold_range_at(from, to, 0, |count, value| {
                visited.push(value);
                if count == fail_at {
                    Err(count)
                } else {
                    Ok(count + 1)
                }
            });
            assert_eq!(result, Err(fail_at));
            assert_eq!(visited, expected[..=fail_at]);
        }
    }
}

#[test]
fn folds_match_materialization_for_all_optimized_views_and_published_bounds() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let source = stored(
        &db,
        "source",
        (0..64_u64).map(|i| StoredU64::from(i * (i + 1))),
    );
    let starts = stored(&db, "starts", (0_usize..64).map(|i| Height::from(i / 3)));
    let days = common::first_heights("days", (0..8usize).map(|i| Height::from(i * 8)));
    let cumulative = stored(
        &db,
        "cumulative",
        (0..64_u64).map(|i| StoredU64::from((i + 1) * 10)),
    );
    let first = stored(&db, "first", (0_usize..64).map(|i| Height::from(i / 2)));
    let count = LazyIndexCountVec::new("count", Version::ONE, &first, &source);
    let delta = LazyPreviousDeltaVec::new("delta", Version::ONE, &source);
    let lookback = LazyLookbackVec::new(
        "lookback",
        Version::ONE,
        &source,
        12,
        |current, previous| current - previous.unwrap_or_default(),
    );
    let since = LazySinceDayVec::new(
        "since",
        Version::ONE,
        &source,
        &days,
        Day1::from(2),
        |current, previous| current - previous,
    );
    let window = |inclusive| {
        LazyWindowVec::new(
            "window",
            Version::ONE,
            &source,
            &starts,
            inclusive,
            |current, previous, count| current - previous + StoredU64::from(count),
        )
    };
    let ratio = LazyIndexedVec::new(
        "ratio",
        Version::ONE,
        &cumulative,
        &source,
        |_, count, numerator| RatioU64::<PartsPerMillion32>::apply(numerator, count),
    );
    let rolling = LazyRollingRatioVec::<
        StoredU64,
        StoredU64,
        PartsPerMillion32,
        ReverseOperands<RatioU64<PartsPerMillion32>>,
    >::new("rolling", Version::ONE, &cumulative, &source, &starts);
    let cached_rolling = LazyRollingRatioVec::<
        StoredU64,
        StoredU64,
        PartsPerMillion32,
        RatioU64<PartsPerMillion32>,
    >::new(
        "cached_rolling",
        Version::ONE,
        &source,
        &cumulative,
        &starts,
    );

    let check = || {
        check_folds(&delta);
        check_folds(&lookback);
        check_folds(&since);
        check_folds(&window(false));
        check_folds(&window(true));
        check_folds(&count);
        check_folds(&ratio);
        check_folds(&rolling);
        check_folds(&cached_rolling);
    };
    check();
    let mut bounds = ReadBounds::new();
    bounds.set("height", 29);
    bounds.scope(check);
}

#[test]
fn fallible_fold_stops_transforming_after_the_first_error() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let source = stored(&db, "source", (0..64_u64).map(StoredU64::from));
    let starts = stored(&db, "starts", (0..64).map(|_| Height::ZERO));
    let calls = Arc::new(AtomicUsize::new(0));
    let captured_calls = calls.clone();
    let window = LazyWindowVec::new(
        "window",
        Version::ONE,
        &source,
        &starts,
        true,
        move |current, _, _| {
            captured_calls.fetch_add(1, Ordering::Relaxed);
            current
        },
    );
    let result =
        window.try_fold_range_at(
            0,
            64,
            0,
            |count, _| if count == 2 { Err(()) } else { Ok(count + 1) },
        );
    assert_eq!(result, Err(()));
    assert_eq!(calls.load(Ordering::Relaxed), 3);
}

#[allow(dead_code)]
#[path = "common/cache.rs"]
mod test_cache;
