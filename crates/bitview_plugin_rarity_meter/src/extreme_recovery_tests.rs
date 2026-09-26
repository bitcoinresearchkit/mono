#[cfg(test)]
use crate::test_common::init_cache;
use brk_types::StoredF32;
use tempfile::tempdir;
use vecdb::AnyVec;

use super::*;
use crate::test_common as common;

fn check(extreme: &Extreme<StoredF32>, len: usize) {
    for threshold in [
        &extreme.thresholds.threshold_pct0_1,
        &extreme.thresholds.threshold_pct0_05,
        &extreme.thresholds.threshold_pct0_025,
    ] {
        assert_eq!(threshold.height.len(), len);
        assert_eq!(threshold.height.collect().len(), len);
    }
    assert_eq!(extreme.tail.ppm.height.len(), len);
    assert_eq!(extreme.tail.ppm.height.collect().len(), len);
    assert_eq!(extreme.rank.height.len(), len);
    assert_eq!(extreme.rank.height.collect().len(), len);
}

#[test]
fn zero_append_truncation_survives_reopen_for_every_extreme_model() {
    init_cache();
    let exit = Exit::new();
    for config in [REALIZED, COINS_IN_LOSS, SELLER_EXHAUSTION] {
        for starting_height in [3usize, 99] {
            let directory = tempdir().unwrap();
            for (previous_len, len) in [(0, 5), (5, 3), (3, 5), (5, 0), (0, 5)] {
                {
                    let db = Database::open(directory.path()).unwrap();
                    let indexes = common::indexes(&db);
                    let mut extreme =
                        Extreme::forced_import(&db, "extreme", Version::ONE, &indexes).unwrap();
                    check(&extreme, previous_len);
                    let mut source = common::stored::<Height, StoredF32>(&db, "source", []);
                    source.truncate_if_needed_at(len).unwrap();
                    for i in source.len()..len {
                        source.push(StoredF32::from((i + 1) as f32));
                    }
                    source.write().unwrap();
                    // Shrinking to the source end leaves no observations to append.
                    extreme
                        .compute(Height::from(starting_height), &source, config, &exit)
                        .unwrap();
                    check(&extreme, len);
                    assert!(extreme.history.is_current(len, config.window));
                }
                let db = Database::open(directory.path()).unwrap();
                let indexes = common::indexes(&db);
                let extreme =
                    Extreme::forced_import(&db, "extreme", Version::ONE, &indexes).unwrap();
                check(&extreme, len); // No compute or write after reopening.
            }
        }
    }
}
