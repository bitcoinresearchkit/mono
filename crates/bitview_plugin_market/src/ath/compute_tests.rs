use bitview_transforms::DaysToYears;
use bitview_vecs::{LazyAggVec, import_cached};
use brk_exit::Exit;
use brk_types::{Cents, Day1, Height, StoredF32, StoredU32, Timestamp, Version};
use rangeindex::SharedRangeMap;
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Database, LazyVec, MapOption, ReadableCloneableVec, ReadableVec, WritableVec,
};

use super::{SecondsToDays, compute_seconds_since};
use crate::test_cache::init_cache;

#[test]
fn elapsed_time_is_exact_across_batches_restarts_and_ath_retests() {
    let budget = init_cache();
    // Include equal timestamps, a retest of the same ATH, a long drawdown whose
    // f32 day count cannot preserve individual seconds, and a new ATH.
    let times = [
        0, 600, 1200, 1200, 1801, 86_488_201, 86_488_202, 86_488_203, 86_488_204, 86_488_204,
        86_488_805,
    ];
    let prices = [100, 90, 90, 100, 90, 90, 90, 90, 110, 110, 90];
    let expected: [u32; 11] = [
        0, 600, 1200, 0, 601, 86_487_001, 86_487_002, 86_487_003, 0, 0, 601,
    ];

    for ends in [vec![11], (1..=11).collect(), vec![2, 5, 6, 8, 11]] {
        let directory = tempdir().unwrap();
        let mut from = 0;
        for end in ends {
            // Reopen all storage for every update: no in-memory ATH state survives.
            let db = Database::open(directory.path()).unwrap();
            let mut price = import_cached::<Height, Cents>(&db, "price", Version::ONE).unwrap();
            let mut timestamp =
                import_cached::<Height, Timestamp>(&db, "timestamp", Version::ONE).unwrap();
            let mut high = import_cached::<Height, Cents>(&db, "high", Version::ONE).unwrap();
            let mut seconds =
                import_cached::<Height, StoredU32>(&db, "seconds", Version::ONE).unwrap();
            for i in from..end {
                price.push(Cents::new(prices[i]));
                timestamp.push(Timestamp::new(1_600_000_000 + times[i]));
            }
            price.write().unwrap();
            timestamp.write().unwrap();
            let exit = Exit::default();
            high.compute_all_time_high(Height::from(from), &price, &exit)
                .unwrap();
            compute_seconds_since(
                &mut seconds,
                Height::from(from),
                &high,
                &price,
                &timestamp,
                &exit,
            )
            .unwrap();
            high.write().unwrap();
            seconds.write().unwrap();
            budget.clear();
            assert_eq!(
                seconds.collect(),
                expected[..end]
                    .iter()
                    .copied()
                    .map(StoredU32::from)
                    .collect::<Vec<_>>()
            );
            from = end;
        }

        let db = Database::open(directory.path()).unwrap();
        let seconds = import_cached::<Height, StoredU32>(&db, "seconds", Version::ONE).unwrap();
        let days = LazyVec::transformed::<SecondsToDays>(
            "days_since_price_ath",
            Version::TWO,
            seconds.read_only_boxed_clone(),
        );
        let expected_days: Vec<_> = expected
            .iter()
            .map(|&seconds| StoredF32::from(seconds as f64 / 86400.0))
            .collect();
        assert_eq!(days.collect(), expected_days);

        // Exercise the same sparse day aggregation followed by the unit transform
        // as the API, including empty days and multiple blocks in a day.
        let daily_seconds = LazyAggVec::<Day1, Option<StoredU32>, Height, StoredU32>::new(
            "seconds",
            Version::TWO,
            seconds.read_only_boxed_clone(),
            SharedRangeMap::new([0usize, 0, 2, 4, 4, 8].map(Height::from).to_vec()),
        );
        let daily_days = LazyVec::transformed::<MapOption<SecondsToDays>>(
            "days_since_price_ath",
            Version::TWO,
            daily_seconds.read_only_boxed_clone(),
        );
        let expected_daily = vec![
            None,
            Some(expected_days[1]),
            Some(expected_days[3]),
            None,
            Some(expected_days[7]),
            Some(expected_days[10]),
        ];
        assert_eq!(daily_days.collect(), expected_daily);
        let daily_years = LazyVec::transformed::<MapOption<DaysToYears>>(
            "years_since_price_ath",
            Version::TWO,
            daily_days.read_only_boxed_clone(),
        );
        assert_eq!(
            daily_years.collect(),
            expected_daily
                .into_iter()
                .map(|days| days.map(|days| StoredF32::from(*days / 365.0)))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn elapsed_time_recomputes_rewritten_and_truncated_history() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut price = import_cached::<Height, Cents>(&db, "price", Version::ONE).unwrap();
    let mut timestamp = import_cached::<Height, Timestamp>(&db, "timestamp", Version::ONE).unwrap();
    let mut high = import_cached::<Height, Cents>(&db, "high", Version::ONE).unwrap();
    let mut seconds = import_cached::<Height, StoredU32>(&db, "seconds", Version::ONE).unwrap();
    let exit = Exit::default();

    for (from, values, expected) in [
        (0, vec![100, 90, 110, 90, 90], vec![0u32, 600, 0, 600, 1200]),
        (2, vec![90, 90, 90], vec![0, 600, 1200, 1800, 2400]),
        (3, vec![100, 90], vec![0, 600, 1200, 0, 600]),
        (3, vec![], vec![0, 600, 1200]),
        (0, vec![], vec![]),
        (0, vec![100, 90], vec![0, 600]),
    ] {
        price.truncate_if_needed_at(from).unwrap();
        timestamp.truncate_if_needed_at(from).unwrap();
        for (i, value) in values.into_iter().enumerate() {
            price.push(Cents::new(value));
            timestamp.push(Timestamp::new(1_600_000_000 + (from + i) as u32 * 600));
        }
        price.write().unwrap();
        timestamp.write().unwrap();
        high.compute_all_time_high(Height::from(from), &price, &exit)
            .unwrap();
        compute_seconds_since(
            &mut seconds,
            Height::from(from),
            &high,
            &price,
            &timestamp,
            &exit,
        )
        .unwrap();
        high.write().unwrap();
        seconds.write().unwrap();
        assert_eq!(
            seconds.collect(),
            expected
                .into_iter()
                .map(StoredU32::from)
                .collect::<Vec<_>>()
        );
    }
}
