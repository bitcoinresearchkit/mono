use common::init_cache;
mod common;

use bitview_vecs::LazySpotValuePerBlock;
use brk_types::{BoundedRatio, Cents, Height, Sats, Version};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, Database, ReadableVec, WritableVec};

#[test]
fn lazy_sides_preserve_stored_rounding_and_follow_source_rewrites() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = common::indexes(&db);
    indexes.first_height.day1 =
        common::first_heights("daily_first_height", [0usize, 2, 4].map(Height::from));
    let inputs = [
        (Sats::from(123_456_789_u64), BoundedRatio::from(0.321)),
        (Sats::from(1_u64), BoundedRatio::from(0.5)),
        (Sats::from(99_u64), BoundedRatio::ONE),
        (Sats::from(99_u64), BoundedRatio::ZERO),
        (Sats::from(99_u64), BoundedRatio::NAN),
    ];
    // Integer division is independent of the production floating-point split.
    // Each side floors separately, so one sat at half weight yields (0, 0).
    let expected = inputs.map(|(supply, weight)| {
        if weight.is_nan() {
            (Sats::ZERO, Sats::ZERO)
        } else {
            let scale = u128::from(BoundedRatio::SCALE);
            let numerator = u128::from(weight.inner());
            (
                Sats::from(supply.as_u128() * numerator / scale),
                Sats::from(supply.as_u128() * (scale - numerator) / scale),
            )
        }
    });
    let mut supply = common::stored::<Height, _>(&db, "supply", inputs.map(|(s, _)| s));
    let mut weights = common::stored::<Height, _>(&db, "weights", inputs.map(|(_, w)| w));
    let spot = common::stored::<Height, _>(&db, "spot", [Cents::from(1_000_000_u64); 5]);
    let weighted = LazySpotValuePerBlock::from_weighted_supply::<false>(
        "awake_supply",
        Version::ONE,
        &supply,
        &weights,
        &indexes,
        &spot,
    );
    let complement = LazySpotValuePerBlock::from_weighted_supply::<true>(
        "dormant_supply",
        Version::ONE,
        &supply,
        &weights,
        &indexes,
        &spot,
    );
    for (height, &(awake, dormant)) in expected.iter().enumerate() {
        assert_eq!(weighted.sats.height.collect_one_at(height), Some(awake));
        assert_eq!(complement.sats.height.collect_one_at(height), Some(dormant));
    }
    // Stock resolutions select the final block, including the partial day.
    for (day, height) in [1usize, 3, 4].into_iter().enumerate() {
        assert_eq!(
            weighted.sats.day1.collect_one_at(day),
            Some(Some(expected[height].0))
        );
        assert_eq!(
            complement.sats.day1.collect_one_at(day),
            Some(Some(expected[height].1))
        );
    }
    supply.truncate_if_needed_at(4).unwrap();
    weights.truncate_if_needed_at(4).unwrap();
    supply.push(Sats::from(101_u64));
    weights.push(BoundedRatio::ONE);
    supply.write().unwrap();
    weights.write().unwrap();
    assert_eq!(
        weighted.sats.height.collect_one_at(4),
        Some(Sats::from(101_u64))
    );
    assert_eq!(complement.sats.height.collect_one_at(4), Some(Sats::ZERO));
    assert_eq!(
        weighted.sats.day1.collect_one_at(2),
        Some(Some(Sats::from(101_u64)))
    );
    assert_eq!(
        complement.sats.day1.collect_one_at(2),
        Some(Some(Sats::ZERO))
    );
}
