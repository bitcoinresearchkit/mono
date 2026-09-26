use bitview_traversable::TreeNode;
use bitview_vecs::{DailyView, RepeatDay};
use brk_types::StoredF32;
use tempfile::tempdir;
use vecdb::{AnyStoredVec, WritableVec};

use super::*;
use crate::{
    START_HEIGHT, inner,
    test_common::{self as common, init_cache},
};

#[test]
fn combined_meter_refreshes_blocks_before_a_daily_median_resume() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let day_start = START_HEIGHT + 1;
    let mut indexes = common::indexes(&db);
    indexes.first_height.day1 =
        common::first_heights("days", [Height::ZERO, Height::from(day_start)]);
    let mapping = common::stored(
        &db,
        "days_by_height",
        (0..day_start + 3).map(|height| Day1::from(usize::from(height >= day_start))),
    );
    let mut daily = common::stored(&db, "daily_median", [Cents::new(100), Cents::new(200)]);
    let source =
        DailyView::<Height, Cents, RepeatDay>::new("daily", Version::ONE, &daily, &mapping);
    let spot = common::stored(&db, "spot", vec![Cents::new(400); day_start + 3]);
    let mut median =
        MedianComponent::forced_import_daily(&db, "median", Version::ONE, &indexes, &source)
            .unwrap();
    let mut meter = inner::forced_import(&db, "full_v2", Version::ONE, &indexes).unwrap();
    let floors: [[&DailyView<Height, Cents, RepeatDay>; 5]; 0] = [];
    let exit = Exit::new();
    median.compute(&Lengths::default(), &spot, &exit).unwrap();
    inner::compute(
        &mut meter,
        &[&median.component],
        &floors,
        &spot,
        Height::ZERO,
        &exit,
    )
    .unwrap();
    let before = meter.prices.pct95.cents.height.collect_one_at(day_start);

    daily.truncate_if_needed_at(1).unwrap();
    daily.push(Cents::new(400));
    daily.write().unwrap();
    let resume = Lengths {
        height: Height::from(day_start + 2),
        ..Lengths::default()
    };
    median.compute(&resume, &spot, &exit).unwrap();
    let start = median.starting_height(resume.height);
    assert_eq!(start, Height::from(day_start));
    inner::compute(
        &mut meter,
        &[&median.component],
        &floors,
        &spot,
        start,
        &exit,
    )
    .unwrap();
    assert_ne!(
        meter.prices.pct95.cents.height.collect_one_at(day_start),
        before
    );

    let mut clean = inner::forced_import(&db, "clean", Version::ONE, &indexes).unwrap();
    inner::compute(
        &mut clean,
        &[&median.component],
        &floors,
        &spot,
        Height::ZERO,
        &exit,
    )
    .unwrap();
    for (actual, expected) in meter.prices.iter().zip(clean.prices.iter()) {
        assert_eq!(
            actual
                .cents
                .height
                .collect_range_at(day_start, day_start + 3),
            expected
                .cents
                .height
                .collect_range_at(day_start, day_start + 3),
        );
    }
    for (actual, expected) in [(&meter.index, &clean.index), (&meter.score, &clean.score)] {
        assert_eq!(
            actual.height.collect_range_at(day_start, day_start + 3),
            expected.height.collect_range_at(day_start, day_start + 3),
        );
    }
}

#[test]
fn daily_median_revisions_refresh_the_whole_day_and_its_bands() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = common::indexes(&db);
    indexes.first_height.day1 =
        common::first_heights("days", [Height::ZERO, Height::from(START_HEIGHT)]);
    let mapping = common::stored(
        &db,
        "days_by_height",
        (0..START_HEIGHT + 3).map(|height| Day1::from(usize::from(height >= START_HEIGHT))),
    );
    let mut daily = common::stored(&db, "daily_median", [Cents::new(100), Cents::new(200)]);
    let source =
        DailyView::<Height, Cents, RepeatDay>::new("daily", Version::ONE, &daily, &mapping);
    let spot = common::stored(&db, "spot", vec![Cents::new(400); START_HEIGHT + 3]);
    let mut median =
        MedianComponent::forced_import_daily(&db, "median", Version::ONE, &indexes, &source)
            .unwrap();
    let exit = Exit::new();
    median.compute(&Lengths::default(), &spot, &exit).unwrap();
    assert_eq!(
        median
            .relative
            .ratio
            .height
            .collect_one_at(START_HEIGHT)
            .unwrap(),
        StoredF32::from(2.0)
    );

    daily.truncate_if_needed_at(1).unwrap();
    daily.push(Cents::new(100));
    daily.write().unwrap();
    let resume = Lengths {
        height: Height::from(START_HEIGHT + 2),
        ..Lengths::default()
    };
    median.compute(&resume, &spot, &exit).unwrap();
    for height in START_HEIGHT..START_HEIGHT + 3 {
        assert_eq!(
            median.relative.ratio.height.collect_one_at(height).unwrap(),
            StoredF32::from(4.0)
        );
        assert_eq!(
            median
                .component
                .bands
                .pct50
                .ratio
                .ratio
                .height
                .collect_one_at(height)
                .unwrap(),
            StoredF32::from(4.0)
        );
        assert_eq!(
            median
                .component
                .bands
                .pct50
                .price
                .cents
                .height
                .collect_one_at(height),
            Some(Cents::new(400))
        );
    }

    // A missing daily snapshot contributes no ratio observation.
    daily.truncate_if_needed_at(1).unwrap();
    daily.write().unwrap();
    median.compute(&resume, &spot, &exit).unwrap();
    assert!(
        median
            .relative
            .ratio
            .height
            .collect_one_at(START_HEIGHT)
            .unwrap()
            .is_nan()
    );

    let TreeNode::Branch(branch) = median.to_tree_node() else {
        panic!("expected median branch")
    };
    for name in ["usd", "cents", "ratio", "pct50"] {
        assert!(branch.contains_key(name), "missing {name}");
    }
}
