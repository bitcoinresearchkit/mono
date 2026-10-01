use bitview_cohort::{AgeAggregateId, AgeRange};
use bitview_urpd::OriginUrpd;
use brk_types::{BoundedRatio, Cents};

use super::{
    CumulativeBucket, Levels, LossPercentileId, MODE_COUNT, ModeId, ModeResult, Modes, Percentiles,
    PriceBands, Thresholds, WeightedModeId, WeightedModes,
};

const LEVEL_PERCENTILES: Levels<f64> = Levels {
    pct10: 0.1,
    pct20: 0.2,
    pct30: 0.3,
    pct40: 0.4,
    pct50: 0.5,
    pct60: 0.6,
    pct70: 0.7,
    pct80: 0.8,
    pct90: 0.9,
};

pub(crate) struct BlockResult {
    pub by_mode: Modes<ModeResult>,
}

impl BlockResult {
    pub fn from_thresholds(thresholds: &Thresholds) -> Self {
        Self {
            by_mode: Modes::from_fn(|mode| ModeResult {
                supply_in_loss_threshold: match thresholds.select(mode) {
                    Some(values) => Percentiles::from_fn(|percentile| {
                        BoundedRatio::from(*percentile.select(values))
                    }),
                    None => Percentiles::from_fn(|_| BoundedRatio::NAN),
                },
                prices: PriceBands::from_fn(|_| Cents::NAN),
            }),
        }
    }

    pub fn evaluate(
        &mut self,
        source: &OriginUrpd,
        weights: &WeightedModes<Option<&AgeRange<f64>>>,
        scratch: &mut Vec<CumulativeBucket>,
    ) {
        let weights = WeightedModeId::ALL.map(|id| *weights.select(id));
        scratch.clear();
        let mut totals = [0_u64; MODE_COUNT];
        for bucket in source.project(&weights, [AgeAggregateId::All.age_range_ids()]) {
            totals[0] += u64::from(bucket.raw[0]);
            for (total, sats) in totals[1..].iter_mut().zip(bucket.weighted[0]) {
                *total += u64::from(sats);
            }
            scratch.push(CumulativeBucket {
                price: bucket.price,
                supplies: totals,
            });
        }
        for (index, mode) in ModeId::ALL.into_iter().enumerate() {
            Self::evaluate_mode(self.by_mode.select_mut(mode), scratch, index);
        }
    }

    fn evaluate_mode(result: &mut ModeResult, buckets: &[CumulativeBucket], mode: usize) {
        let thresholds = &result.supply_in_loss_threshold;
        if thresholds.iter().all(|threshold| threshold.is_nan()) {
            return;
        }
        let Some(last) = buckets.last() else {
            return;
        };
        let denominator = last.supplies[mode];
        let zero_cost = buckets
            .first()
            .filter(|b| b.price.inner() == 0)
            .map_or(0, |b| b.supplies[mode]);
        if denominator == zero_cost {
            return;
        }

        let mut p95_floor = None;
        result.prices.floor = Percentiles::from_fn(|percentile| {
            let threshold = f64::from(*percentile.select(thresholds));
            if threshold.is_nan() {
                return Cents::NAN;
            }
            let floor = buckets.partition_point(|bucket| {
                let supply = bucket.supplies[mode];
                // Keep the published bounded threshold and the original division order.
                supply == 0 || (denominator - supply) as f64 / denominator as f64 > threshold
            });
            buckets.get(floor).map_or(Cents::NAN, |bucket| {
                if percentile == LossPercentileId::Pct95 {
                    p95_floor = Some(floor);
                }
                Cents::from(bucket.price)
            })
        });
        if let Some(floor) = p95_floor {
            // Exclude only buckets preceding the floor; the floor itself belongs to the tail.
            let before = floor
                .checked_sub(1)
                .map_or(0, |i| buckets[i].supplies[mode]);
            let total = denominator - before;
            let tail = &buckets[floor..];
            result.prices.level = Levels::from_fn(|percentile| {
                let target = total as f64 * *percentile.select(&LEVEL_PERCENTILES);
                let index = tail
                    .partition_point(|bucket| ((bucket.supplies[mode] - before) as f64) < target);
                tail.get(index)
                    .map_or(Cents::NAN, |bucket| Cents::from(bucket.price))
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::array;

    use bitview_cohort::AgeRange;
    use bitview_urpd::OriginUrpd;
    use brk_types::{BoundedRatio, Cents, CentsCompact, Sats, Timestamp};
    use statedb::{Amount, State};

    use super::{BlockResult, LEVEL_PERCENTILES};
    use crate::{
        CumulativeBucket, Levels, LossPercentileId, MODE_COUNT, ModeId, Percentiles, PriceBandId,
        Thresholds, WeightedModes,
    };

    fn cumulative(
        entries: impl IntoIterator<Item = (CentsCompact, Sats)>,
    ) -> Vec<CumulativeBucket> {
        let mut total = 0;
        entries
            .into_iter()
            .map(|(price, sats)| {
                total += u64::from(sats);
                CumulativeBucket {
                    price,
                    supplies: [total; MODE_COUNT],
                }
            })
            .collect()
    }

    fn evaluate<const N: usize>(result: &mut BlockResult, entries: [(u32, u64); N]) {
        let entries =
            cumulative(entries.map(|(price, sats)| (CentsCompact::new(price), Sats::new(sats))));
        for (index, mode) in ModeId::ALL.into_iter().enumerate() {
            BlockResult::evaluate_mode(result.by_mode.select_mut(mode), &entries, index);
        }
    }

    #[test]
    fn history_views_match_materialized_buckets_with_missing_weights() {
        let state = State::new(
            [7, 12].map(|sats| Amount { sats, count: 1 }).to_vec(),
            [0; 32],
        )
        .unwrap();
        let source = OriginUrpd::new(
            &state,
            &[Cents::new(100), Cents::new(200)],
            &[Timestamp::new(0), Timestamp::new(3600)],
        )
        .unwrap();
        let weights = AgeRange::from_fn(|_| 0.5);
        let thresholds = Thresholds::from_fn(|_| Some(Percentiles::from_fn(|_| 0.5)));
        let mut scratch = Vec::new();
        for available in [Some(&weights), None, Some(&weights)] {
            let mut actual = BlockResult::from_thresholds(&thresholds);
            actual.evaluate(
                &source,
                &WeightedModes::from_fn(|_| available),
                &mut scratch,
            );
            for mode in ModeId::ALL {
                let buckets: &[(u32, u64)] = match (mode.weighted(), available) {
                    (None, _) => &[(100, 7), (200, 12)],
                    (Some(_), Some(_)) => &[(100, 3), (200, 6)],
                    (Some(_), None) => &[],
                };
                let mut expected = BlockResult::from_thresholds(&thresholds);
                let expected_buckets = cumulative(
                    buckets
                        .iter()
                        .map(|&(price, sats)| (CentsCompact::new(price), Sats::new(sats))),
                );
                BlockResult::evaluate_mode(
                    expected.by_mode.select_mut(mode),
                    &expected_buckets,
                    mode as usize,
                );
                for id in PriceBandId::ALL {
                    assert_eq!(
                        id.select(&actual.by_mode.select(mode).prices),
                        id.select(&expected.by_mode.select(mode).prices),
                    );
                }
            }
        }
    }

    #[test]
    fn calibrated_loss_share_sets_floor_and_levels() {
        let urpds = [(100, 50), (200, 50)];
        let thresholds = Thresholds::from_fn(|_| Some(Percentiles::from_fn(|_| 0.5)));
        let mut result = BlockResult::from_thresholds(&thresholds);
        evaluate(&mut result, urpds);
        let result = &result.by_mode.coinflow;

        assert_eq!(
            result.supply_in_loss_threshold,
            Percentiles::from_fn(|_| BoundedRatio::from(0.5))
        );
        assert_eq!(
            result.prices.floor,
            Percentiles::from_fn(|_| Cents::new(100))
        );
        assert_eq!(
            result.prices.level,
            Levels {
                pct10: Cents::new(100),
                pct20: Cents::new(100),
                pct30: Cents::new(100),
                pct40: Cents::new(100),
                pct50: Cents::new(100),
                pct60: Cents::new(200),
                pct70: Cents::new(200),
                pct80: Cents::new(200),
                pct90: Cents::new(200),
            }
        );
    }

    #[test]
    fn zero_cost_distribution_stays_missing() {
        let urpds = [(0, 100)];
        let thresholds = Thresholds::from_fn(|_| Some(Percentiles::from_fn(|_| 1.0)));
        let mut result = BlockResult::from_thresholds(&thresholds);
        evaluate(&mut result, urpds);
        assert!(result.by_mode.raw.prices.floor.pct95.is_nan());
    }

    #[test]
    fn conditional_supply_includes_the_floor_bucket() {
        let urpds = [(0, 2), (100, 8), (150, 0), (200, 30), (300, 60)];
        let thresholds = Thresholds::from_fn(|_| Some(Percentiles::from_fn(|_| 0.65)));
        let mut result = BlockResult::from_thresholds(&thresholds);
        evaluate(&mut result, urpds);
        for mode in result.by_mode.iter() {
            assert_eq!(mode.prices.floor.pct95, Cents::new(200));
            // The conditional supply is 90, including 30 at the floor.
            assert_eq!(mode.prices.level.pct30, Cents::new(200));
            assert_eq!(mode.prices.level.pct40, Cents::new(300));
        }
    }

    #[test]
    fn floors_use_the_same_bounded_threshold_that_is_published() {
        let urpds = [(100, 9), (200, 1)];
        let thresholds = Thresholds::from_fn(|_| Some(Percentiles::from_fn(|_| 0.1)));
        let mut result = BlockResult::from_thresholds(&thresholds);
        assert!(f64::from(result.by_mode.raw.supply_in_loss_threshold.pct95) < 0.1);
        evaluate(&mut result, urpds);
        assert_eq!(result.by_mode.raw.prices.floor.pct95, Cents::new(200));
        let missing = BlockResult::from_thresholds(&Thresholds::from_fn(|_| None));
        assert!(missing.by_mode.raw.supply_in_loss_threshold.pct95.is_nan());
    }
    fn reference(result: &mut BlockResult, entries: &[(u32, u64)]) {
        let denominator: u64 = entries.iter().map(|&(_, sats)| sats).sum();
        if denominator == 0 || !entries.iter().any(|&(p, s)| p != 0 && s != 0) {
            return;
        }
        for mode in result.by_mode.iter_mut() {
            let mut floor95 = None;
            for percentile in LossPercentileId::ALL {
                let threshold = f64::from(*percentile.select(&mode.supply_in_loss_threshold));
                let mut remaining = denominator;
                for (i, &(price, sats)) in entries.iter().enumerate() {
                    if sats == 0 {
                        continue;
                    }
                    remaining -= sats;
                    if remaining as f64 / denominator as f64 <= threshold {
                        *percentile.select_mut(&mut mode.prices.floor) =
                            Cents::new(u64::from(price));
                        if percentile == LossPercentileId::Pct95 {
                            floor95 = Some(i);
                        }
                        break;
                    }
                }
            }
            if let Some(floor) = floor95 {
                let tail = &entries[floor..];
                let total: u64 = tail.iter().map(|&(_, s)| s).sum();
                mode.prices.level = Levels::from_fn(|percentile| {
                    let target = total as f64 * *percentile.select(&LEVEL_PERCENTILES);
                    let mut sum = 0;
                    for &(price, sats) in tail {
                        if sats == 0 {
                            continue;
                        }
                        sum += sats;
                        if sum as f64 >= target {
                            return Cents::new(u64::from(price));
                        }
                    }
                    Cents::NAN
                });
            }
        }
    }

    #[test]
    fn cumulative_search_matches_linear_rules_at_plateaus_and_float_boundaries() {
        let mut seed = 23_u64;
        for scale in [1, 3, 1_000_001, (1_u64 << 53) / 8191, u64::MAX / 1_000_000] {
            for case in 0..64 {
                let entries: [(u32, u64); 64] = array::from_fn(|i| {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let sats = if i % 3 == case % 3 {
                        0
                    } else {
                        (seed >> 32) % 8191 * scale
                    };
                    ((i * 100) as u32, sats)
                });
                let thresholds = Thresholds::from_fn(|mode| {
                    (case % 7 != mode as usize).then(|| {
                        Percentiles::from_fn(|id| {
                            [0.0, 1.0, 0.1, 0.65, f64::NAN, (case as f64 + 0.5) / 64.0]
                                [(id as usize + case) % 6]
                        })
                    })
                });
                let mut expected = BlockResult::from_thresholds(&thresholds);
                reference(&mut expected, &entries);
                let mut actual = BlockResult::from_thresholds(&thresholds);
                evaluate(&mut actual, entries);
                for mode in ModeId::ALL {
                    for id in PriceBandId::ALL {
                        assert_eq!(
                            id.select(&actual.by_mode.select(mode).prices),
                            id.select(&expected.by_mode.select(mode).prices),
                            "scale={scale} case={case} mode={mode:?} band={id:?}"
                        );
                    }
                }
            }
        }
    }
}
