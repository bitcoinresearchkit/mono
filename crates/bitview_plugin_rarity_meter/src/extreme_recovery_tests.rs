use brk_types::StoredF32;
use tempfile::tempdir;
use vecdb::AnyVec;

use super::*;
use crate::test_common::{self as common, init_cache};

fn check(
    extreme: &Extreme<StoredF32>,
    values: &[StoredF32],
    upper: bool,
    positive_only: bool,
    rolling: bool,
) {
    let thresholds = [
        &extreme.thresholds.threshold_pct0_1.height,
        &extreme.thresholds.threshold_pct0_05.height,
        &extreme.thresholds.threshold_pct0_025.height,
    ];
    for threshold in thresholds {
        assert_eq!(threshold.len(), values.len());
    }
    assert_eq!(extreme.tail.ppm.height.len(), values.len());
    assert_eq!(extreme.rank.height.len(), values.len());
    for height in MIN_HISTORY_BLOCKS.saturating_sub(1)..values.len() {
        let value = f64::from(values[height]);
        let accepted = |value: f64| value.is_finite() && (!positive_only || value > 0.0);
        let mut history: Vec<_> = values[..height]
            .iter()
            .copied()
            .map(f64::from)
            .filter(|&value| accepted(value))
            .collect();
        if rolling {
            history.drain(..history.len().saturating_sub(MIN_HISTORY_BLOCKS));
        }
        history.sort_unstable_by(f64::total_cmp);
        let available = accepted(value) && history.len() >= MIN_HISTORY_BLOCKS;
        let expected = [0.001, 0.0005, 0.00025].map(|tail| {
            if available {
                let percentile = if upper { 1.0 - tail } else { tail };
                history[((history.len() - 1) as f64 * percentile).floor() as usize]
            } else {
                f64::NAN
            }
        });
        for (threshold, expected) in thresholds.into_iter().zip(expected) {
            let actual = *threshold.collect_one_at(height).unwrap();
            assert!(actual == *StoredF32::from(expected) || (actual.is_nan() && expected.is_nan()));
        }
        let tail = if available {
            let count = history
                .iter()
                .filter(|&&past| if upper { past >= value } else { past <= value })
                .count();
            (count + 1) as f64 / (history.len() + 1) as f64
        } else {
            f64::NAN
        };
        assert_eq!(
            extreme.tail.ppm.height.collect_one_at(height),
            Some(PartsPerMillion32::from(tail))
        );
        let rank = expected
            .iter()
            .rposition(|&boundary| {
                if upper {
                    value >= boundary
                } else {
                    value <= boundary
                }
            })
            .map_or(0, |index| index as u8 + 1);
        assert_eq!(
            extreme.rank.height.collect_one_at(height),
            Some(StoredU8::new(rank))
        );
    }
}

#[test]
fn event_values_survive_backfill_rollback_resume_and_reopen() {
    init_cache();
    let exit = Exit::new();
    for (config, upper, positive_only, rolling) in [
        (REALIZED, true, false, false),
        (COINS_IN_LOSS, true, true, false),
        (SELLER_EXHAUSTION, false, true, true),
    ] {
        let directory = tempdir().unwrap();
        let end = MIN_HISTORY_BLOCKS + 9;
        let mut values = Vec::new();
        for (phase, len) in [end, end - 2, end, 0, end].into_iter().enumerate() {
            let previous_len = values.len();
            values.truncate(len);
            // The first observation equals the 0.025% boundary, so rank must be inclusive.
            let boundary =
                if upper { MIN_HISTORY_BLOCKS - 53 } else { 53 } as f32 / MIN_HISTORY_BLOCKS as f32;
            let tail = if phase == 0 {
                [
                    boundary,
                    f32::NAN,
                    2.0,
                    0.0,
                    0.5,
                    0.0004,
                    0.00075,
                    0.9996,
                    0.99925,
                ]
            } else {
                [
                    0.0002,
                    f32::NAN,
                    1.5,
                    -0.5,
                    0.2,
                    0.00045,
                    0.0008,
                    0.99965,
                    0.9993,
                ]
            };
            for height in values.len()..len {
                values.push(StoredF32::from(if height < MIN_HISTORY_BLOCKS {
                    (height + 1) as f32 / MIN_HISTORY_BLOCKS as f32
                } else {
                    tail[height - MIN_HISTORY_BLOCKS]
                }));
            }
            {
                let db = Database::open(directory.path()).unwrap();
                let indexes = common::indexes(&db);
                let mut extreme =
                    Extreme::forced_import(&db, "extreme", Version::ONE, &indexes).unwrap();
                assert_eq!(extreme.rank.height.len(), previous_len);
                let mut source = common::stored::<Height, StoredF32>(&db, "source", []);
                source.truncate_if_needed_at(len).unwrap();
                for &value in &values[source.len()..] {
                    source.push(value);
                }
                source.write().unwrap();
                // Clamping to the source end must persist even a rollback with no append.
                extreme
                    .compute(Height::from(len + 99), &source, config, &exit)
                    .unwrap();
                check(&extreme, &values, upper, positive_only, rolling);
                assert!(extreme.history.is_current(len, config.window));
            }
            let db = Database::open(directory.path()).unwrap();
            let indexes = common::indexes(&db);
            let extreme = Extreme::forced_import(&db, "extreme", Version::ONE, &indexes).unwrap();
            check(&extreme, &values, upper, positive_only, rolling);
        }
    }
}
