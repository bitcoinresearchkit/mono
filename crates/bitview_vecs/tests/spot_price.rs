use bitview_transforms::{CentsUnsignedToDollars, CentsUnsignedToSats};
use bitview_vecs::{OhlcPrice, SplitPrice, SpotPrice};
use brk_types::{Cents, Height, Version};
use common::init_cache;
use tempfile::tempdir;
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableVec, UnaryTransform, WritableVec};

mod common;

#[test]
fn shared_price_shapes_preserve_integer_units_inverse_extrema_empty_periods_and_rewrites() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = common::indexes(&db);
    indexes.first_height.day1 =
        common::first_heights("day_starts", [0usize, 2, 2, 4].map(Height::from));
    let mut spot = SpotPrice::forced_import(&db, "price", Version::new(31), &indexes).unwrap();
    for value in [200u64, 400, 100, 800, 300, 600] {
        spot.cents.height.push(Cents::from(value));
    }
    spot.cents.height.write().unwrap();
    let ohlc = OhlcPrice::from_spot("price_ohlc", Version::new(31), &indexes, &spot);
    let split = SplitPrice::new("price", Version::new(31), &indexes, &spot, &ohlc);

    assert_eq!(spot.cents.height.name(), "price_cents");
    assert_eq!(split.high.sats.day1.name(), "price_high_sats");
    for (i, (open, high, low, close)) in [
        (200u64, 400u64, 200u64, 400u64),
        (400, 400, 400, 400),
        (100, 800, 100, 800),
        (300, 600, 300, 600),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            split.open.cents.day1.collect_one_at(i),
            Some(Cents::from(open))
        );
        assert_eq!(
            split.high.cents.day1.collect_one_at(i),
            Some(Cents::from(high))
        );
        assert_eq!(
            split.low.cents.day1.collect_one_at(i),
            Some(Cents::from(low))
        );
        assert_eq!(
            split.close.cents.day1.collect_one_at(i),
            Some((i != 1).then(|| Cents::from(close)))
        );
        assert_eq!(
            split.high.sats.day1.collect_one_at(i),
            Some(CentsUnsignedToSats::apply(Cents::from(low)))
        );
        assert_eq!(
            split.low.sats.day1.collect_one_at(i),
            Some(CentsUnsignedToSats::apply(Cents::from(high)))
        );
        let candle = ohlc.sats.day1.collect_one_at(i).unwrap();
        assert_eq!(*candle.high, CentsUnsignedToSats::apply(Cents::from(low)));
        assert_eq!(*candle.low, CentsUnsignedToSats::apply(Cents::from(high)));
    }
    assert_eq!(
        spot.usd.height.collect_one_at(0),
        Some(CentsUnsignedToDollars::apply(Cents::from(200u64)))
    );
    assert_eq!(
        spot.sats.height.collect_one_at(0),
        Some(CentsUnsignedToSats::apply(Cents::from(200u64)))
    );

    spot.cents.height.truncate_if_needed_at(4).unwrap();
    for value in [900u64, 1000] {
        spot.cents.height.push(Cents::from(value));
    }
    spot.cents.height.write().unwrap();
    assert_eq!(
        split.low.cents.day1.collect_one_at(3),
        Some(Cents::from(900u64))
    );
    assert_eq!(
        split.high.sats.day1.collect_one_at(3),
        Some(CentsUnsignedToSats::apply(Cents::from(900u64)))
    );
    assert_eq!(
        split.close.cents.day1.collect_one_at(3),
        Some(Some(Cents::from(1000u64)))
    );
}
