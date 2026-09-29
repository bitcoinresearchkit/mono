use brk_types::StoredF32;
use tempfile::tempdir;
use vecdb::{AnyStoredVec, Budgeted, EagerVec, PcoVec, WritableVec};

use super::*;
use crate::{
    START_HEIGHT, inner,
    test_common::{self as common, init_cache},
};

#[test]
fn median_revisions_start_at_the_changed_block_and_match_a_clean_meter() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let start = START_HEIGHT + 1;
    let end = start + 3;
    let mut source = common::stored(&db, "median_source", vec![Cents::new(200); end]);
    let spot = common::stored(&db, "spot", vec![Cents::new(400); end]);
    let mut median =
        MedianComponent::forced_import(&db, "median", Version::ONE, &indexes, &source).unwrap();
    let mut meter = inner::forced_import(&db, "meter", Version::ONE, &indexes).unwrap();
    let floors: [[&EagerVec<PcoVec<Height, Cents, Budgeted>>; 5]; 0] = [];
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
    let prefix = meter
        .prices
        .pct95
        .cents
        .height
        .collect_range_at(START_HEIGHT, start);

    source.truncate_if_needed_at(start).unwrap();
    for value in [Cents::new(100), Cents::NAN, Cents::new(400)] {
        source.push(value);
    }
    source.write().unwrap();
    let resume = Lengths {
        height: Height::from(start),
        ..Lengths::default()
    };
    median.compute(&resume, &spot, &exit).unwrap();
    inner::compute(
        &mut meter,
        &[&median.component],
        &floors,
        &spot,
        resume.height,
        &exit,
    )
    .unwrap();
    assert_eq!(
        median.relative.ratio.height.collect_one_at(start - 1),
        Some(StoredF32::from(2.0))
    );
    assert_eq!(
        median.relative.ratio.height.collect_one_at(start),
        Some(StoredF32::from(4.0))
    );
    assert!(
        median
            .relative
            .ratio
            .height
            .collect_one_at(start + 1)
            .unwrap()
            .is_nan()
    );
    assert_eq!(
        median.relative.ratio.height.collect_one_at(start + 2),
        Some(StoredF32::from(1.0))
    );
    assert_eq!(
        prefix,
        meter
            .prices
            .pct95
            .cents
            .height
            .collect_range_at(START_HEIGHT, start)
    );

    let mut clean_median =
        MedianComponent::forced_import(&db, "clean_median", Version::ONE, &indexes, &source)
            .unwrap();
    clean_median
        .compute(&Lengths::default(), &spot, &exit)
        .unwrap();
    let mut clean = inner::forced_import(&db, "clean", Version::ONE, &indexes).unwrap();
    inner::compute(
        &mut clean,
        &[&clean_median.component],
        &floors,
        &spot,
        Height::ZERO,
        &exit,
    )
    .unwrap();
    for (actual, expected) in meter.prices.iter().zip(clean.prices.iter()) {
        assert_eq!(
            actual.cents.height.collect_range_at(START_HEIGHT, end),
            expected.cents.height.collect_range_at(START_HEIGHT, end)
        );
    }
    for (actual, expected) in [(&meter.index, &clean.index), (&meter.score, &clean.score)] {
        assert_eq!(
            actual.height.collect_range_at(START_HEIGHT, end),
            expected.height.collect_range_at(START_HEIGHT, end)
        );
    }
}
