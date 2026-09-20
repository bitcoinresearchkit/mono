use bitview_cohort::UTXOAggregate;
use bitview_vecs::{
    CachedSeries, DailyMappings, LazyDailyPriceWithRatio, RangeMapLookupVec, import_cached,
};
use brk_types::{Cents, Day1, Height, PriceRatio, Version};
use common::init_cache;
use tempfile::tempdir;
use vecdb::{
    AnySerializableVec, AnyStoredVec, AnyVec, Database, ReadableCloneableVec, ReadableVec,
    WritableVec,
};

#[path = "daily_price/benchmark.rs"]
mod benchmark;
mod common;

#[test]
fn daily_price_sources_persist_and_expose_prices_ratios_and_aligned_rewrites() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = common::indexes(&db);
    indexes.height_day1 = common::stored::<Height, _>(
        &db,
        "test_height_day",
        [0usize, 0, 1, 1, 2, 2].map(Day1::from),
    )
    .read_only_boxed_clone();
    indexes.first_height.day1 =
        common::first_heights("test_first_height", [0usize, 2, 4].map(Height::from));
    indexes.height_day1 =
        RangeMapLookupVec::new(indexes.first_height.day1.mapping(), &indexes.height_day1)
            .read_only_boxed_clone();
    let spot = common::stored::<Height, _>(
        &db,
        "test_spot",
        [200, 300, 400, 500, 1_000_000, 1_000_000].map(Cents::new),
    )
    .read_only_boxed_clone();
    let mappings = DailyMappings::new(&indexes);
    let import = || {
        let stored = UTXOAggregate::try_from_fn(|id| {
            import_cached::<Day1, Cents>(
                &db,
                &id.metric_name("test_capitalized_price_cents"),
                Version::ONE,
            )
        })
        .unwrap();
        let prices = UTXOAggregate::from_fn(|id| {
            LazyDailyPriceWithRatio::from_day1_source(
                &id.metric_name("test_capitalized_price"),
                Version::ONE,
                id.select(&stored),
                &indexes,
                &mappings,
                &spot,
            )
        });
        (stored, prices)
    };
    let push = |stored: &mut UTXOAggregate<CachedSeries<Day1, Cents>>,
                values: UTXOAggregate<Cents>| {
        for (target, &value) in stored.iter_mut().zip(values.iter()) {
            target.push(value);
        }
    };
    let (mut stored, prices) = import();
    for (all, sth, lth) in [
        (Cents::new(100), 50, 200),
        (Cents::NAN, 75, 300),
        (Cents::new(1), 1, 1),
    ] {
        push(
            &mut stored,
            UTXOAggregate {
                all,
                sth: Cents::new(sth),
                lth: Cents::new(lth),
            },
        );
    }
    for target in stored.iter_mut() {
        target.write().unwrap();
    }
    let mut json = Vec::new();
    prices
        .all
        .usd
        .day1
        .write_json(Some(0), Some(3), &mut json)
        .unwrap();
    assert_eq!(json, b"[1.0,null,0.01]");
    assert_eq!(
        prices.all.relative.ppm.height.collect_one_at(0),
        Some(PriceRatio::from(2.0))
    );
    assert_eq!(
        prices.all.relative.ppm.height.collect_one_at(1),
        Some(PriceRatio::from(3.0))
    );
    assert!(
        prices
            .all
            .relative
            .ppm
            .height
            .collect_one_at(2)
            .unwrap()
            .is_nan()
    );
    assert_eq!(
        prices.all.relative.ppm.height.collect_one_at(4),
        Some(PriceRatio::MAX)
    );
    assert_eq!(
        prices.all.relative.ppm.day1.collect_one_at(0).flatten(),
        Some(PriceRatio::from(3.0))
    );
    assert!(
        prices
            .all
            .relative
            .ppm
            .day1
            .collect_one_at(1)
            .flatten()
            .unwrap()
            .is_nan()
    );
    assert_eq!(
        prices.sth.relative.ppm.height.collect_one_at(0),
        Some(PriceRatio::from(4.0))
    );
    assert_eq!(
        prices.lth.relative.ppm.height.collect_one_at(0),
        Some(PriceRatio::ONE)
    );
    drop(prices);
    drop(stored);
    let (mut stored, prices) = import();
    assert_eq!(
        prices.all.cents.day1.collect_one_at(0),
        Some(Cents::new(100))
    );
    // Stored sources invalidate their changed suffix during truncation.
    for target in stored.iter_mut() {
        target.truncate_if_needed_at(1).unwrap();
    }
    push(
        &mut stored,
        UTXOAggregate {
            all: Cents::new(200),
            sth: Cents::new(40),
            lth: Cents::new(600),
        },
    );
    push(
        &mut stored,
        UTXOAggregate {
            all: Cents::new(50),
            sth: Cents::new(25),
            lth: Cents::new(100),
        },
    );
    for target in stored.iter_mut() {
        target.write().unwrap();
    }
    assert_eq!(
        prices.all.relative.ppm.height.collect_one_at(2),
        Some(PriceRatio::from(2.0))
    );
    assert_eq!(
        prices.all.cents.day1.collect_one_at(0),
        Some(Cents::new(100))
    );
    for price in prices.iter() {
        assert_eq!(price.cents.day1.len(), 3);
    }
    assert_eq!(
        prices.sth.cents.day1.collect_one_at(1),
        Some(Cents::new(40))
    );
    assert_eq!(
        prices.lth.cents.day1.collect_one_at(1),
        Some(Cents::new(600))
    );
}
