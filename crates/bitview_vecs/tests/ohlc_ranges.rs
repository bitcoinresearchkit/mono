#![cfg(feature = "diagnostics")]

use bitview_vecs::LazyOhlcVec;
use brk_types::{Cents, Day1, Height, OHLCCents, Version};
use common::init_cache;
use rangeindex::SharedRangeMap;
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Budgeted, Database, EagerVec, ImportableVec, PcoVec, ReadableVec, WritableVec,
    diagnostics,
};

#[allow(dead_code)]
mod common;

fn values(candle: &OHLCCents) -> (u64, u64, u64, u64) {
    (**candle.open, **candle.high, **candle.low, **candle.close)
}

#[test]
fn cold_candles_batch_their_price_span_and_warm_reads_reuse_it() {
    let budget = init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();

    let mut prices =
        EagerVec::<PcoVec<Height, Cents, Budgeted>>::import(&db, "prices", Version::ONE).unwrap();
    let mut boundaries = Vec::new();
    for i in 0..32_768usize {
        prices.push(Cents::from(100 + i as u64));
        if i % 128 == 0 {
            boundaries.push(Height::from(i));
        }
    }
    prices.write().unwrap();
    let candles = LazyOhlcVec::<Day1>::new(
        "candles",
        Version::TWO,
        &prices,
        SharedRangeMap::new(boundaries),
    );
    diagnostics::take();
    let actual = candles.collect_range_at(4, 20);
    assert_eq!(diagnostics::take(), 3, "decode the bounded price span once");
    assert!(!prices.read_cached_into_at(0, 32_768, &mut Vec::new()));
    assert_eq!(
        candles
            .collect_range_at(4, 20)
            .iter()
            .map(values)
            .collect::<Vec<_>>(),
        actual.iter().map(values).collect::<Vec<_>>()
    );
    assert_eq!(diagnostics::take(), 0);
    for (day, candle) in (4..20usize).zip(&actual) {
        assert_eq!(**candle.open, 100 + (day * 128) as u64);
        assert_eq!(**candle.close, 100 + ((day + 1) * 128 - 1) as u64);
        assert_eq!(candle.high.inner(), candle.close.inner());
        assert_eq!(candle.low.inner(), candle.open.inner());
    }
    budget.clear();
    diagnostics::take();
    let selected = candles.read_sorted_at(&[4, 19]);
    assert_eq!(
        selected.iter().map(values).collect::<Vec<_>>(),
        [values(&actual[0]), values(&actual[15])]
    );
    assert_eq!(
        diagnostics::take(),
        2,
        "sparse candles do not read the intervening days"
    );
}
