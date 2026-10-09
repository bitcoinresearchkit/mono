use bitview_cohort::{AgeAggregateId, AgeRange};
use bitview_primitives::BoundedRatio;
use bitview_urpd::OriginUrpd;
use brk_types::Cents;

use super::{
    CumulativeBucket, Levels, LossPercentileId, MODE_COUNT, ModeId, ModeResult, Modes, Percentiles,
    PriceBands, Thresholds, WeightedModeId, WeightedModes,
};

const LEVEL_PERCENTILES: Levels<f64> = Levels {
    pct10: 0.1,
    pct20: 0.2,
    pct30: 0.3,
    pct40: 0.4,
    median: 0.5,
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
