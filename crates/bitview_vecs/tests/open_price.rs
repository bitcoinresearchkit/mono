use std::{hint::black_box, time::Instant};

use bitview_transforms::{CentsUnsignedToDollars, CentsUnsignedToSats, OhlcCentsToOpenCents};
use bitview_vecs::{OhlcPrice, SplitPrice, SpotPrice};
use brk_types::{Cents, Height, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Database, LazyVec, ReadBounds, ReadableCloneableVec, ReadableVec,
    UnaryTransform, WritableVec,
};

#[cfg(feature = "diagnostics")]
use vecdb::diagnostics;

use common::init_cache;

mod common;

#[test]
fn open_prices_match_candles_through_empty_periods_publication_and_rewrites() {
    let budget = init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = common::indexes(&db);
    let starts = [0usize, 0, 0, 2, 2, 4, 6, 6, 8];
    indexes.first_height.day1 = common::first_heights("days", starts.map(Height::from));
    indexes.first_height.epoch = common::first_heights("epochs", starts.map(Height::from));
    let mut spot = SpotPrice::forced_import(&db, "price", Version::ONE, &indexes).unwrap();
    let candles = OhlcPrice::from_spot("ohlc", Version::ONE, &indexes, &spot);
    let split = SplitPrice::new("price", Version::ONE, &indexes, &spot, &candles);
    let reader = split.clone();
    assert_eq!(reader.open.cents.day1.name(), "price_open_cents");
    assert_eq!(reader.open.usd.day1.name(), "price_open");
    assert_eq!(reader.open.sats.day1.name(), "price_open_sats");

    for prices in [
        vec![200, 400, 100, 800, 300, 600],
        vec![900, 400, 700, 800, 500, 600],
        vec![900, 400, 700, 800, 500, 600, 400, 1000, 900, 500],
        vec![900, 400, 700],
        vec![],
        vec![0, u64::MAX, 100, 800, 300, 600],
    ] {
        spot.cents.height.truncate_if_needed_at(0).unwrap();
        for &price in &prices {
            spot.cents.height.push(if price == u64::MAX {
                Cents::NAN
            } else {
                Cents::new(price)
            });
        }
        spot.cents.height.write().unwrap();

        for visible in 0..=prices.len() {
            let mut bounds = ReadBounds::new();
            bounds.set("height", visible);
            bounds.scope(|| {
                let expected: Vec<_> = candles
                    .cents
                    .day1
                    .collect()
                    .into_iter()
                    .map(|candle| *candle.open)
                    .collect();
                let expected_usd: Vec<_> = expected
                    .iter()
                    .copied()
                    .map(CentsUnsignedToDollars::apply)
                    .collect();
                // Whole sats cannot represent a missing price; test those
                // inputs in cents/USD and leave their conversion contract intact.
                let expected_sats = expected.iter().all(|price| !price.is_nan()).then(|| {
                    expected
                        .iter()
                        .copied()
                        .map(CentsUnsignedToSats::apply)
                        .collect::<Vec<_>>()
                });
                for cold in [false, true] {
                    if cold {
                        budget.clear();
                    }
                    assert_eq!(reader.open.cents.day1.collect(), expected);
                    assert_eq!(reader.open.cents.epoch.collect(), expected);
                    assert_eq!(reader.open.usd.day1.collect(), expected_usd);
                    if let Some(expected_sats) = &expected_sats {
                        assert_eq!(&reader.open.sats.day1.collect(), expected_sats);
                    }
                    for (i, &price) in expected.iter().enumerate() {
                        assert_eq!(reader.open.cents.day1.collect_one_at(i), Some(price));
                    }
                    assert_eq!(reader.open.cents.day1.collect_one_at(expected.len()), None);
                    let requested = [0, 2, 2, 4, 8, 8, 9, usize::MAX];
                    assert_eq!(
                        reader.open.cents.day1.read_sorted_at(&requested),
                        requested
                            .iter()
                            .filter_map(|&i| expected.get(i).copied())
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(
                        reader.open.cents.day1.collect_range_at(2, 6),
                        expected[2..6]
                    );
                    let mut appended = vec![Cents::new(42)];
                    reader.open.cents.day1.read_into_at(2, 6, &mut appended);
                    assert_eq!(&appended[1..], &expected[2..6]);
                    assert_eq!(appended[0], Cents::new(42));
                    let folded =
                        reader
                            .open
                            .cents
                            .day1
                            .fold_range_at(0, 9, Vec::new(), |mut out, price| {
                                out.push(price);
                                out
                            });
                    assert_eq!(folded, expected);
                    let stopped = reader
                        .open
                        .cents
                        .day1
                        .try_fold_range_at(0, 9, 0, |count, _| {
                            if count == 2 {
                                Err(count)
                            } else {
                                Ok(count + 1)
                            }
                        });
                    assert_eq!(stopped, Err(2));
                }
            });
        }
    }

    // The existing readers follow the writer-owned boundary mapping after reorgs.
    indexes
        .first_height
        .day1
        .mapping()
        .update_at(0, [0usize, 0, 1, 1, 3, 3, 5, 9].map(Height::from));
    let expected: Vec<_> = candles
        .cents
        .day1
        .collect()
        .into_iter()
        .map(|candle| *candle.open)
        .collect();
    assert_eq!(reader.open.cents.day1.collect(), expected);

    #[cfg(feature = "diagnostics")]
    {
        spot.cents.height.truncate_if_needed_at(0).unwrap();
        for i in 0..32_768_u64 {
            spot.cents.height.push(Cents::new(100 + i));
        }
        spot.cents.height.write().unwrap();
        indexes
            .first_height
            .day1
            .mapping()
            .update_at(0, [0usize, 16_384].map(Height::from));
        budget.clear();
        diagnostics::take();
        assert_eq!(
            reader.open.cents.day1.collect_one_at(0),
            Some(Cents::new(100))
        );
        assert_eq!(diagnostics::take(), 1, "open reads one physical price page");
        budget.clear();
        diagnostics::take();
        assert_eq!(
            *candles.cents.day1.collect_one_at(0).unwrap().open,
            Cents::new(100)
        );
        assert!(diagnostics::take() > 1, "full candles read the price span");
    }
}

#[test]
#[ignore = "synthetic before/after read-path benchmark"]
fn benchmark_open_price_reads() {
    let budget = init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = common::indexes(&db);
    const BLOCKS: usize = 966_505;
    const DAYS: usize = 6463;
    indexes.first_height.day1 = common::first_heights(
        "days",
        (0..DAYS).map(|day| Height::from(day * BLOCKS / DAYS)),
    );
    let mut spot = SpotPrice::forced_import(&db, "price", Version::ONE, &indexes).unwrap();
    for i in 0..BLOCKS {
        spot.cents
            .height
            .push(Cents::new(10_000 + (i as u64 * 37) % 1_000_000));
    }
    spot.cents.height.write().unwrap();
    let candles = OhlcPrice::from_spot("ohlc", Version::ONE, &indexes, &spot);
    let split = SplitPrice::new("price", Version::ONE, &indexes, &spot, &candles);
    let old = LazyVec::transformed::<OhlcCentsToOpenCents>(
        "old_open",
        Version::ONE,
        candles.cents.day1.read_only_boxed_clone(),
    );
    let new = &split.open.cents.day1;
    assert_eq!(old.collect(), new.collect());
    let selected: Vec<_> = (0..DAYS).step_by(7).collect();
    for (case, indices) in [
        ("range", Vec::new()),
        ("full", (0..DAYS).collect::<Vec<_>>()),
        ("weekly", selected),
        ("latest", vec![DAYS - 1]),
    ] {
        for cold in [false, true] {
            for (name, source) in [
                ("ohlc", &old as &dyn ReadableVec<_, _>),
                ("open", new as &dyn ReadableVec<_, _>),
            ] {
                let mut times = Vec::new();
                for _ in 0..21 {
                    if cold {
                        budget.clear();
                    }
                    let start = Instant::now();
                    black_box(if case == "range" {
                        source.collect_range_dyn(0, DAYS)
                    } else {
                        source.read_sorted_at(&indices)
                    });
                    times.push(start.elapsed());
                }
                times.sort_unstable();
                eprintln!(
                    "{case} {name} retention_cold={cold}: {:?}",
                    times[times.len() / 2]
                );
            }
        }
    }
}
