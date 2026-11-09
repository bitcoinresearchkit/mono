use bitview_primitives::BoundedRatio;
use bitview_urpd::OriginUrpd;
use brk_types::Cents;

use super::{
    CumulativeBucket, Levels, LossPercentileId, ModeId, ModeResult, ModeWeights, Modes,
    Percentiles, PriceBands, Thresholds,
};

const LEVEL_PERCENTILES: Levels<f64> = Levels {
    p10: 0.1,
    p20: 0.2,
    p30: 0.3,
    p40: 0.4,
    median: 0.5,
    p60: 0.6,
    p70: 0.7,
    p80: 0.8,
    p90: 0.9,
};

pub(crate) type BlockResult = Modes<ModeResult>;

impl BlockResult {
    pub(crate) fn from_thresholds(thresholds: &Thresholds) -> Self {
        Self::from_fn(|mode| ModeResult {
            supply_in_loss_threshold: match thresholds.select(mode) {
                Some(values) => Percentiles::from_fn(|percentile| {
                    BoundedRatio::from(*values.select(percentile))
                }),
                None => Percentiles::from_fn(|_| BoundedRatio::NAN),
            },
            prices: PriceBands::from_fn(|_| Cents::NAN),
        })
    }

    /// Each mode's age-range weights; unweighted supply is always raw.
    pub(crate) fn evaluate(
        &mut self,
        source: &OriginUrpd,
        weights: &ModeWeights<'_>,
        scratch: &mut Vec<CumulativeBucket>,
    ) {
        scratch.clear();
        let mut totals = Modes::<u64>::default();
        for (price, supplies) in weights.project(source) {
            for &mode in ModeId::ALL {
                *totals.select_mut(mode) += *supplies.select(mode);
            }
            scratch.push(CumulativeBucket {
                price,
                supplies: totals,
            });
        }
        for &mode in ModeId::ALL {
            Self::evaluate_mode(self.select_mut(mode), scratch, mode);
        }
    }

    /// `result`'s thresholds are all defined or all undefined (NaN).
    fn evaluate_mode(result: &mut ModeResult, buckets: &[CumulativeBucket], mode: ModeId) {
        let thresholds = &result.supply_in_loss_threshold;
        if thresholds.iter().all(|threshold| threshold.is_nan()) {
            return;
        }
        let Some(last) = buckets.last() else {
            return;
        };
        let denominator = *last.supplies.select(mode);
        let zero_cost = buckets
            .first()
            .filter(|b| b.price.inner() == 0)
            .map_or(0, |b| *b.supplies.select(mode));
        if denominator == zero_cost {
            return;
        }

        // The last bucket holds the whole denominator, so no share exceeds a threshold there and
        // every search below ends inside its slice.
        let mut p95_floor = 0;
        result.prices.floor = Percentiles::from_fn(|percentile| {
            let threshold = f64::from(*thresholds.select(percentile));
            let floor = buckets.partition_point(|bucket| {
                let supply = *bucket.supplies.select(mode);
                // The bounded threshold as published, and this division order: both decide
                // rounding at the boundary.
                supply == 0 || (denominator - supply) as f64 / denominator as f64 > threshold
            });
            if percentile == LossPercentileId::P95 {
                p95_floor = floor;
            }
            Cents::from(buckets[floor].price)
        });
        // Exclude only buckets preceding the floor; the floor itself belongs to the tail.
        let before = p95_floor
            .checked_sub(1)
            .map_or(0, |i| *buckets[i].supplies.select(mode));
        let total = denominator - before;
        let tail = &buckets[p95_floor..];
        result.prices.cost_basis_above_p95_floor = Levels::from_fn(|percentile| {
            let target = total as f64 * *LEVEL_PERCENTILES.select(percentile);
            let index = tail.partition_point(|bucket| {
                ((*bucket.supplies.select(mode) - before) as f64) < target
            });
            Cents::from(tail[index].price)
        });
    }
}
