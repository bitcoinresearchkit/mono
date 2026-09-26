use bitview_vecs::{DailyView, RepeatDay};
use brk_types::{Day1, PartsPerMillion32};
use tempfile::tempdir;

use super::*;
use crate::test_common::{self as common, init_cache};

#[test]
fn expanded_v2_scores_cap_overflowing_totals() {
    init_cache();
    let directory = tempdir().unwrap();
    {
        let db = Database::open(directory.path()).unwrap();
        let indexes = common::indexes(&db);
        let spot = common::stored(&db, "spot", [1, 1000, 100].map(Cents::new));
        let reference = common::stored(&db, "reference", [Cents::new(100); 3]);
        let daily_floor = common::stored(&db, "daily_floor", [Cents::new(100)]);
        let days = common::stored(&db, "days", [Day1::from(0usize); 3]);
        let floor =
            DailyView::<Height, Cents, RepeatDay>::new("floor", Version::ONE, &daily_floor, &days);
        let mut component =
            component::forced_import(&db, "component", Version::ONE, &indexes, &reference).unwrap();
        for ratio in component.ratios.iter_mut() {
            for _ in 0..3 {
                ratio.push(PartsPerMillion32::new(1_000_000));
            }
            ratio.write().unwrap();
        }
        let mut local = forced_import(&db, "local_v2", Version::ONE, &indexes).unwrap();
        let mut cycle = forced_import(&db, "cycle_v2", Version::ONE, &indexes).unwrap();
        let mut full = forced_import(&db, "full_v2", Version::ONE, &indexes).unwrap();
        let exit = Exit::new();
        let floors = [[&floor; 5]; 3];
        compute(
            &mut local,
            &[&component; 8],
            &floors[..0],
            &spot,
            Height::ZERO,
            &exit,
        )
        .unwrap();
        compute(
            &mut cycle,
            &[&component; 16],
            &floors,
            &spot,
            Height::ZERO,
            &exit,
        )
        .unwrap();
        compute_combined(&mut full, &[&local, &cycle], &spot, Height::ZERO, &exit).unwrap();
        assert_scores(&full, [-128, 120, 0]);

        // Direct aggregation must use the same score range as combining meters.
        let mut direct = forced_import(&db, "direct", Version::ONE, &indexes).unwrap();
        compute(
            &mut direct,
            &[&component; 24],
            &floors,
            &spot,
            Height::ZERO,
            &exit,
        )
        .unwrap();
        assert_scores(&direct, [-128, 120, 0]);
        compute(
            &mut direct,
            &[&component; 27],
            &floors[..0],
            &spot,
            Height::ZERO,
            &exit,
        )
        .unwrap();
        assert_scores(&direct, [-128, 127, 0]);
        db.flush().unwrap();
    }
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let full = forced_import(&db, "full_v2", Version::ONE, &indexes).unwrap();
    assert_scores(&full, [-128, 120, 0]);
}

fn assert_scores(meter: &RarityMeterInner, expected: [i8; 3]) {
    let actual: Vec<i8> = meter
        .score
        .height
        .collect()
        .iter()
        .map(|value| **value)
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn combined_scores_cap_only_after_summing() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let mut meters =
        ["a", "b", "c"].map(|name| forced_import(&db, name, Version::ONE, &indexes).unwrap());
    for (meter, values) in
        meters
            .iter_mut()
            .zip([[100, -100, 100], [100, -100, 100], [-100, 100, 100]])
    {
        for value in values {
            meter.score.height.push(StoredI8::new(value));
        }
        meter.score.height.write().unwrap();
    }
    let mut full = forced_import(&db, "full", Version::ONE, &indexes).unwrap();
    full.compute_combined_score(&meters.each_ref(), Height::ZERO, &Exit::new())
        .unwrap();
    assert_scores(&full, [100, -100, 127]);
}

#[test]
fn uncapped_scores_are_reset_without_resetting_the_index() {
    init_cache();
    let directory = tempdir().unwrap();
    {
        let db = Database::open(directory.path()).unwrap();
        let indexes = common::indexes(&db);
        let version = Version::ONE + VERSION;
        let mut score = PerBlock::forced_import(&db, "meter_score", version, &indexes).unwrap();
        score.height.push(StoredI8::new(-100));
        score.height.write().unwrap();
        let mut index = PerBlock::forced_import(&db, "meter_index", version, &indexes).unwrap();
        index.height.push(StoredI8::new(-5));
        index.height.write().unwrap();
        db.flush().unwrap();
    }
    {
        let db = Database::open(directory.path()).unwrap();
        let indexes = common::indexes(&db);
        let mut meter = forced_import(&db, "meter", Version::ONE, &indexes).unwrap();
        assert_eq!(meter.score.height.len(), 0);
        assert_eq!(
            meter.index.height.collect_one_at(0),
            Some(StoredI8::new(-5))
        );
        meter.score.height.push(StoredI8::new(-128));
        meter.score.height.write().unwrap();
        db.flush().unwrap();
    }
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let meter = forced_import(&db, "meter", Version::ONE, &indexes).unwrap();
    assert_eq!(
        meter.score.height.collect_one_at(0),
        Some(StoredI8::new(-128))
    );
}
