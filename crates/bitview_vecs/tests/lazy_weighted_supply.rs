use common::init_cache;
mod common;

use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_compute::WeightedCohortState;
use bitview_vecs::{CachedSeries, LazySpotValuePerBlock, import_cached};
use brk_types::{BoundedRatio, Cents, Height, Sats, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Database, ImportableVec, PcoVec, ReadableCloneableVec, ReadableVec, WritableVec,
};

#[test]
fn lazy_sides_preserve_stored_rounding_and_follow_source_rewrites() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = common::indexes(&db);
    indexes.first_height.day1 =
        common::first_heights("daily_first_height", [0usize, 2, 4].map(Height::from));
    let mut supply: AgeRange<CachedSeries<Height, Sats>> = AgeRange::from_fn(|id| {
        import_cached(&db, &format!("supply_{}", id.index()), Version::ONE).unwrap()
    });
    let mut weights: AgeRange<CachedSeries<Height, BoundedRatio>> = AgeRange::from_fn(|id| {
        import_cached(&db, &format!("weights_{}", id.index()), Version::ONE).unwrap()
    });
    let mut spot = PcoVec::<Height, Cents>::forced_import(&db, "spot", Version::ONE).unwrap();
    let inputs = [
        (Sats::from(123_456_789_u64), BoundedRatio::from(0.321)),
        (Sats::from(1_u64), BoundedRatio::from(0.5)),
        (Sats::from(99_u64), BoundedRatio::ONE),
        (Sats::from(99_u64), BoundedRatio::ZERO),
        (Sats::from(99_u64), BoundedRatio::NAN),
    ];
    for (s, w) in inputs {
        for v in supply.iter_mut() {
            v.push(s);
        }
        for v in weights.iter_mut() {
            v.push(w);
        }
        spot.push(Cents::from(1_000_000_u64));
    }
    for v in supply.iter_mut() {
        v.write().unwrap();
    }
    for v in weights.iter_mut() {
        v.write().unwrap();
    }
    spot.write().unwrap();
    let spot = spot.read_only_boxed_clone();
    for &id in AgeRangeId::ALL {
        let raw = id.select(&supply).read_only_boxed_clone();
        let weight = id.select(&weights).read_only_boxed_clone();
        let weighted = LazySpotValuePerBlock::from_weighted_supply::<false>(
            "awake_supply",
            Version::ONE,
            &raw,
            &weight,
            &indexes,
            &spot,
        );
        let complement = LazySpotValuePerBlock::from_weighted_supply::<true>(
            "dormant_supply",
            Version::ONE,
            &raw,
            &weight,
            &indexes,
            &spot,
        );
        for (height, (s, w)) in inputs.iter().copied().enumerate() {
            let expected = WeightedCohortState::split_supply(s, w);
            assert_eq!(
                weighted.sats.height.collect_one_at(height),
                Some(expected.0)
            );
            assert_eq!(
                complement.sats.height.collect_one_at(height),
                Some(expected.1)
            );
        }
        // Both floors are zero: subtraction would incorrectly return one sat.
        assert_eq!(weighted.sats.height.collect_one_at(1), Some(Sats::ZERO));
        assert_eq!(complement.sats.height.collect_one_at(1), Some(Sats::ZERO));
        // Stock resolutions select the final block, including the partial day.
        for (day, height) in [1usize, 3, 4].into_iter().enumerate() {
            let expected = WeightedCohortState::split_supply(inputs[height].0, inputs[height].1);
            assert_eq!(
                weighted.sats.day1.collect_one_at(day),
                Some(Some(expected.0))
            );
            assert_eq!(
                complement.sats.day1.collect_one_at(day),
                Some(Some(expected.1))
            );
        }
        for v in supply.iter_mut() {
            v.truncate_if_needed_at(4).unwrap();
        }
        for v in weights.iter_mut() {
            v.truncate_if_needed_at(4).unwrap();
        }
        for v in supply.iter_mut() {
            v.push(Sats::from(101_u64));
        }
        for v in weights.iter_mut() {
            v.push(BoundedRatio::ONE);
        }
        for v in supply.iter_mut() {
            v.write().unwrap();
        }
        for v in weights.iter_mut() {
            v.write().unwrap();
        }
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
        // Restore before validating the next age range.
        for v in supply.iter_mut() {
            v.truncate_if_needed_at(4).unwrap();
        }
        for v in weights.iter_mut() {
            v.truncate_if_needed_at(4).unwrap();
        }
        for v in supply.iter_mut() {
            v.push(inputs[4].0);
        }
        for v in weights.iter_mut() {
            v.push(inputs[4].1);
        }
        for v in supply.iter_mut() {
            v.write().unwrap();
        }
        for v in weights.iter_mut() {
            v.write().unwrap();
        }
    }
}
