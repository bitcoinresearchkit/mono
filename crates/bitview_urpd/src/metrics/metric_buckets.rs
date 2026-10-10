use std::ops::Range;

use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_primitives::{
    CentsCompact, CostBasisByPercentile, PERCENTILES, PERCENTILES_LEN, PartsPerMillion32,
};
use brk_types::Cents;

use bitview_vecs::Density;

use crate::{COST_BASIS_PRICE_DIGITS, ProjectedBucket};

const COHORTS: usize = AgeAggregateId::ALL.len();

/// One block's statistics and densities for one cohort.
pub(super) struct CohortBlock {
    pub cost_basis: CostBasisByPercentile,
    pub supply_density: Density<PartsPerMillion32>,
    pub capital_density: Density<PartsPerMillion32>,
}

/// One block's projected buckets in price order: each bucket's weighted sats per cohort, kept
/// compact so the statistics sweep stays in cache, plus every cohort's totals.
#[derive(Default)]
pub(super) struct MetricBuckets {
    prices: Vec<CentsCompact>,
    sats: Vec<[u64; COHORTS]>,
    totals: [Totals; COHORTS],
    /// The buckets within 5% below and above spot, split at spot; `None` without a valid spot.
    band: Option<(Range<usize>, Range<usize>)>,
}

#[derive(Clone, Copy, Default)]
struct Totals {
    sats: u128,
    value: u128,
}

impl MetricBuckets {
    pub fn update(
        &mut self,
        entries: impl Iterator<Item = ProjectedBucket<1, COHORTS>>,
        spot: Cents,
    ) {
        self.prices.clear();
        self.sats.clear();
        self.totals = [Totals::default(); COHORTS];
        for bucket in entries {
            let price = bucket.price.as_u128();
            let sats = bucket.weighted.map(|[sats]| u64::from(sats));
            for (totals, &sats) in self.totals.iter_mut().zip(&sats) {
                let value = price * u128::from(sats);
                totals.sats += u128::from(sats);
                totals.value += value;
            }
            self.prices.push(bucket.price);
            self.sats.push(sats);
        }
        // Edges are rounded like the bucket prices (and holders' price index): the band takes
        // the buckets of 95% and 105% of spot, and the spot bucket is in profit.
        self.band = spot.finite_inner().filter(|&p| p > 0).map(|spot| {
            let bucket = |cents: f64| {
                u128::from(
                    Cents::from(cents.round() as u64).round_to_significant(COST_BASIS_PRICE_DIGITS),
                )
            };
            let spot = spot as f64;
            let (low, mid, high) = (bucket(spot * 0.95), bucket(spot), bucket(spot * 1.05));
            let lower = self.prices.partition_point(|p| p.as_u128() < low);
            let split = lower + self.prices[lower..].partition_point(|p| p.as_u128() <= mid);
            let upper = split + self.prices[split..].partition_point(|p| p.as_u128() <= high);
            (lower..split, split..upper)
        });
    }

    /// Every cohort's statistics and densities from the last update.
    pub fn block(&self) -> AgeAggregate<CohortBlock> {
        AgeAggregate::from_fn(|id| {
            let cohort = id.index();
            let totals = self.totals[cohort];
            let (supply_density, capital_density) = self.densities(cohort, totals);
            CohortBlock {
                cost_basis: self.cost_basis(cohort, totals),
                supply_density,
                capital_density,
            }
        })
    }

    /// Coin- and capital-weighted percentiles in one sweep: each is the first bucket whose
    /// running total passes its target.
    fn cost_basis(&self, cohort: usize, totals: Totals) -> CostBasisByPercentile {
        let targets = |total: u128| {
            PERCENTILES.map(|percentile| (total * u128::from(percentile) / 100).saturating_sub(1))
        };
        let (coin_targets, dollar_targets) = (targets(totals.sats), targets(totals.value));
        let mut per_coin = [Cents::default(); PERCENTILES_LEN];
        let mut per_dollar = [Cents::default(); PERCENTILES_LEN];
        let (mut coin, mut dollar) = (
            if totals.sats == 0 { PERCENTILES_LEN } else { 0 },
            if totals.value == 0 {
                PERCENTILES_LEN
            } else {
                0
            },
        );
        let (mut sats, mut value) = (0_u128, 0_u128);
        for (&price, row) in self.prices.iter().zip(&self.sats) {
            if coin == PERCENTILES_LEN && dollar == PERCENTILES_LEN {
                break;
            }
            let bucket = u128::from(row[cohort]);
            sats += bucket;
            value += price.as_u128() * bucket;
            while coin < PERCENTILES_LEN && sats > coin_targets[coin] {
                per_coin[coin] = price.into();
                coin += 1;
            }
            while dollar < PERCENTILES_LEN && value > dollar_targets[dollar] {
                per_dollar[dollar] = price.into();
                dollar += 1;
            }
        }
        CostBasisByPercentile {
            per_coin,
            per_dollar,
        }
    }

    /// Weighted supply and invested capital within 5% of spot, split at spot, over the buckets
    /// inside the band.
    fn densities(
        &self,
        cohort: usize,
        totals: Totals,
    ) -> (Density<PartsPerMillion32>, Density<PartsPerMillion32>) {
        let Some((profit, loss)) = &self.band else {
            return (Density::NAN, Density::NAN);
        };
        let sum = |range: &Range<usize>| {
            self.prices[range.clone()]
                .iter()
                .zip(&self.sats[range.clone()])
                .fold((0_u128, 0_u128), |(sats, value), (price, row)| {
                    let bucket = u128::from(row[cohort]);
                    (sats + bucket, value + price.as_u128() * bucket)
                })
        };
        let (profit_sats, profit_value) = sum(profit);
        let (loss_sats, loss_value) = sum(loss);
        (
            Density::from_sums(totals.sats, profit_sats, loss_sats),
            Density::from_sums(totals.value, profit_value, loss_value),
        )
    }
}
