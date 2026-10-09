use std::ops::Range;

use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_primitives::{
    CentsCompact, CostBasisByPercentile, PERCENTILES, PERCENTILES_LEN, PartsPerMillion32,
};
use brk_types::Cents;

use super::density::SupplyDensity;
use crate::ProjectedBucket;

const COHORTS: usize = AgeAggregateId::ALL.len();

/// One block's statistics and supply density for one cohort.
pub(super) struct CohortBlock {
    pub cost_basis: CostBasisByPercentile,
    pub density: SupplyDensity<PartsPerMillion32>,
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
        self.band = spot.finite_inner().filter(|&p| p > 0).map(|spot| {
            let spot = u128::from(spot);
            let lower = self
                .prices
                .partition_point(|p| p.as_u128() * 100 < spot * 95);
            let split = lower + self.prices[lower..].partition_point(|p| p.as_u128() <= spot);
            let upper =
                split + self.prices[split..].partition_point(|p| p.as_u128() * 100 <= spot * 105);
            (lower..split, split..upper)
        });
    }

    /// Every cohort's statistics and supply density from the last update.
    pub fn block(&self) -> AgeAggregate<CohortBlock> {
        AgeAggregate::from_fn(|id| {
            let cohort = id.index();
            let totals = self.totals[cohort];
            CohortBlock {
                cost_basis: self.cost_basis(cohort, totals),
                density: self.density(cohort, totals),
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

    /// Weighted supply within 5% of spot, split at spot, over the buckets inside the band.
    fn density(&self, cohort: usize, totals: Totals) -> SupplyDensity<PartsPerMillion32> {
        let Some((profit, loss)) = &self.band else {
            return SupplyDensity::NAN;
        };
        let sum = |range: &Range<usize>| {
            self.sats[range.clone()]
                .iter()
                .map(|row| u128::from(row[cohort]))
                .sum()
        };
        SupplyDensity::from_sums(totals.sats, sum(profit), sum(loss))
    }
}
