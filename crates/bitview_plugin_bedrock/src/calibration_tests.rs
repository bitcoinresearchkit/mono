use brk_types::{PartsPerMillion32, StoredF64};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, BytesVec, Database, ImportableVec, WritableVec};

use super::*;
#[test]
fn incremental_quantiles_match_sorted_reference_without_current_block() {
    let mut calibration = Calibration {
        end: 0,
        version: Version::ONE,
        histories: Modes::from_fn(|_| ExactOrderStats::new(MINIMUM_BLOCKS + 10)),
    };
    let mut reference = Vec::new();
    for i in 0..MINIMUM_BLOCKS + 10 {
        let value = ((i * 37) % 1009) as f64 / 1009.0;
        let shares = Modes::from_fn(|_| Some(value));
        let thresholds = calibration.thresholds(&shares);
        assert_eq!(thresholds.iter().all(Option::is_none), i < MINIMUM_BLOCKS);
        if i >= MINIMUM_BLOCKS {
            reference.sort_unstable_by(f64::total_cmp);
            for mode in ModeId::ALL {
                for (actual, p) in thresholds
                    .select(mode)
                    .as_ref()
                    .unwrap()
                    .iter()
                    .zip(PERCENTILES.iter())
                {
                    let rank = p * (reference.len() - 1) as f64;
                    let expected = reference[rank.floor() as usize] * (1.0 - rank.fract())
                        + reference[rank.ceil() as usize] * rank.fract();
                    assert_eq!(*actual, expected);
                }
            }
        }
        reference.push(value);
        calibration.observe(shares);
        assert_eq!(calibration.end, i + 1);
    }
}
#[test]
fn missing_observations_advance_height_without_entering_the_sample() {
    let mut calibration = Calibration {
        end: 0,
        version: Version::ONE,
        histories: Modes::from_fn(|_| ExactOrderStats::new(1)),
    };
    calibration.observe(Modes::from_fn(|_| None));
    assert_eq!(calibration.end, 1);
    assert!(calibration.histories.iter().all(ExactOrderStats::is_empty));
}

#[test]
fn restored_block_prefix_matches_live_history_and_excludes_future_values() {
    let root = tempdir().unwrap();
    let db = Database::open(root.path()).unwrap();
    let mut raw =
        BytesVec::<Height, PartsPerMillion32>::forced_import(&db, "raw", Version::ONE).unwrap();
    let mut weighted =
        BytesVec::<Height, StoredF64>::forced_import(&db, "weighted", Version::ONE).unwrap();
    let count = MINIMUM_BLOCKS + 3;
    for i in 0..count {
        let value = (i % 101) as f64 / 100.0;
        raw.push(PartsPerMillion32::from(value));
        weighted.push(StoredF64::from(if i == count - 1 {
            f64::NAN
        } else {
            value
        }));
    }
    raw.write().unwrap();
    weighted.write().unwrap();
    let sources = WeightedModes::from_fn(|_| &weighted as &dyn ReadableVec<Height, StoredF64>);
    let mut live = Calibration::from_sources(&raw, &sources, 0, Version::ONE);
    for i in 0..count {
        let shares = Calibration::loss_shares(&raw, &sources, Height::from(i));
        if i >= MINIMUM_BLOCKS {
            let restored = Calibration::from_sources(&raw, &sources, i, Version::ONE);
            for (actual, expected) in live
                .thresholds(&shares)
                .iter()
                .zip(restored.thresholds(&shares).iter())
            {
                assert_eq!(actual, expected);
            }
        }
        live.observe(shares);
    }
    assert_eq!(live.end, count);
}
