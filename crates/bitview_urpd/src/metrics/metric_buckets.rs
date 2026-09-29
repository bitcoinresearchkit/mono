use std::array;

use bitview_cohort::{AGE_RANGE_COUNT, AgeRange, AgeRangeId, Term, UTXOAggregate};
use brk_types::{Cents, CentsCompact, PartsPerMillion32, Sats};

use super::{density::SupplyDensity, price_distribution::PriceDistribution};
use crate::distribution::AgeCutoffs;

#[derive(Default)]
struct Masses {
    all: f64,
    age: AgeCutoffs<f64>,
    long: f64,
}

impl Masses {
    fn add(&mut self, age: AgeRangeId, sats: u64, weights: &AgeRange<f64>) {
        let mass = sats as f64 * *age.select(weights);
        self.all += mass;
        for value in self.age.containing_mut(age) {
            *value += mass;
        }
        if age.term() == Term::Lth {
            self.long += mass;
        }
    }
}

/// Only percentile calculations retain buckets; density is accumulated directly.
pub(super) struct MetricBuckets {
    pub prices: UTXOAggregate<PriceDistribution>,
    /// All, under 4 months, under 5 months (STH), and under 6 months.
    pub density: [SupplyDensity<PartsPerMillion32>; 4],
}

impl Default for MetricBuckets {
    fn default() -> Self {
        Self {
            prices: UTXOAggregate::default(),
            density: [SupplyDensity::NAN; 4],
        }
    }
}

impl MetricBuckets {
    pub fn update<'a>(
        &mut self,
        entries: impl Iterator<Item = (CentsCompact, &'a [u64; AGE_RANGE_COUNT])>,
        weights: &AgeRange<f64>,
        spot: Cents,
    ) {
        for prices in self.prices.iter_mut() {
            prices.clear();
        }
        let spot = spot.finite_inner().filter(|&p| p > 0).map(u128::from);
        let mut valid = spot.is_some();
        let spot = spot.unwrap_or_default();
        let mut totals = [0_u128; 4];
        let mut profits = [0_u128; 4];
        let mut losses = [0_u128; 4];
        for (price, supplies) in entries {
            let mut bucket = Masses::default();
            for (&age, &sats) in AgeRangeId::ALL.iter().zip(supplies) {
                if sats != 0 {
                    bucket.add(age, sats, weights);
                }
            }
            let sats = [
                bucket.all,
                bucket.age.under_4m,
                bucket.age.under_5m,
                bucket.age.under_6m,
            ]
            .map(|mass| mass.floor() as u64);
            self.prices.all.push(price, Sats::new(sats[0]));
            self.prices.sth.push(price, Sats::new(sats[2]));
            self.prices
                .lth
                .push(price, Sats::new(bucket.long.floor() as u64));
            for (total, sats) in totals.iter_mut().zip(sats) {
                *total += u128::from(sats);
            }
            let Some(price) = price.finite_inner().map(u128::from) else {
                valid = false;
                continue;
            };
            if price * 100 >= spot * 95 && price * 100 <= spot * 105 {
                let target = if price <= spot {
                    &mut profits
                } else {
                    &mut losses
                };
                for (target, sats) in target.iter_mut().zip(sats) {
                    *target += u128::from(sats);
                }
            }
        }
        self.density = if valid {
            array::from_fn(|i| SupplyDensity::from_sums(totals[i], profits[i], losses[i]))
        } else {
            [SupplyDensity::NAN; 4]
        };
    }
}

#[cfg(test)]
#[path = "metric_buckets_tests.rs"]
mod tests;
