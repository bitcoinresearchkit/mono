use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_primitives::{
    CentsCompact, CostBasisByPercentile, PERCENTILES, PERCENTILES_LEN, PartsPerMillion32,
};
use brk_types::Cents;

use bitview_vecs::Density;

use crate::{COST_BASIS_PRICE_DIGITS, DensityBand, ProjectedBucket};

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
    /// The density band's edges as bucket positions: its first bucket, the first bucket above
    /// spot (the split) and the first bucket past it; `None` without a valid spot.
    band: Option<[usize; 3]>,
}

#[derive(Clone, Copy, Default)]
struct Totals {
    sats: u128,
    value: u128,
}

impl Totals {
    fn add(&mut self, price: u128, sats: u64) {
        self.sats += u128::from(sats);
        self.value += price * u128::from(sats);
    }
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
                totals.add(price, sats);
            }
            self.prices.push(bucket.price);
            self.sats.push(sats);
        }
        // Exact edges on the grid prices (as holders' price index cuts them); the spot bucket is
        // in profit.
        self.band = DensityBand::new(spot).map(|band| {
            let first = |(numer, denom): (u128, u128)| {
                self.prices.partition_point(|p| p.as_u128() * denom < numer)
            };
            let mid = u128::from(band.spot().round_to_significant(COST_BASIS_PRICE_DIGITS));
            let lower = first(band.first_inside());
            let upper = first(band.first_above());
            let split = lower + self.prices[lower..upper].partition_point(|p| p.as_u128() <= mid);
            [lower, split, upper]
        });
    }

    /// Every cohort's statistics and densities from the last update, in one sweep over the
    /// buckets: each percentile is the first bucket whose running total reaches its nearest
    /// rank, ceil(percentile% of the total), and the band's sums are differences of the running
    /// totals at its edges.
    pub fn block(&self) -> AgeAggregate<CohortBlock> {
        let targets = |total: u128| {
            PERCENTILES.map(|percentile| {
                (total * u128::from(percentile))
                    .div_ceil(100)
                    .saturating_sub(1)
            })
        };
        let coin_targets = self.totals.map(|totals| targets(totals.sats));
        let dollar_targets = self.totals.map(|totals| targets(totals.value));
        let mut by_coin = [[Cents::NAN; PERCENTILES_LEN]; COHORTS];
        let mut by_dollar = [[Cents::NAN; PERCENTILES_LEN]; COHORTS];
        // Each cohort's next unresolved percentile; all resolved without a total.
        let mut coin = self
            .totals
            .map(|totals| if totals.sats == 0 { PERCENTILES_LEN } else { 0 });
        let mut dollar = self.totals.map(|totals| {
            if totals.value == 0 {
                PERCENTILES_LEN
            } else {
                0
            }
        });
        let edges = self.band.unwrap_or_default();
        let mut at_edges = [[Totals::default(); COHORTS]; 3];
        let mut edge = if self.band.is_some() { 0 } else { 3 };
        let mut running = [Totals::default(); COHORTS];
        for (index, (&price, row)) in self.prices.iter().zip(&self.sats).enumerate() {
            while edge < 3 && edges[edge] == index {
                at_edges[edge] = running;
                edge += 1;
            }
            if edge == 3
                && coin.iter().all(|&next| next == PERCENTILES_LEN)
                && dollar.iter().all(|&next| next == PERCENTILES_LEN)
            {
                break;
            }
            let price_u128 = price.as_u128();
            for cohort in 0..COHORTS {
                let running = &mut running[cohort];
                running.add(price_u128, row[cohort]);
                let coin = &mut coin[cohort];
                while *coin < PERCENTILES_LEN && running.sats > coin_targets[cohort][*coin] {
                    by_coin[cohort][*coin] = price.into();
                    *coin += 1;
                }
                let dollar = &mut dollar[cohort];
                while *dollar < PERCENTILES_LEN && running.value > dollar_targets[cohort][*dollar] {
                    by_dollar[cohort][*dollar] = price.into();
                    *dollar += 1;
                }
            }
        }
        while edge < 3 {
            at_edges[edge] = running;
            edge += 1;
        }
        AgeAggregate::from_fn(|id| {
            let cohort = id.index();
            let totals = self.totals[cohort];
            let (supply_density, capital_density) = if self.band.is_some() {
                let [lower, split, upper] = at_edges.map(|totals| totals[cohort]);
                (
                    Density::from_sums(
                        totals.sats,
                        split.sats - lower.sats,
                        upper.sats - split.sats,
                    ),
                    Density::from_sums(
                        totals.value,
                        split.value - lower.value,
                        upper.value - split.value,
                    ),
                )
            } else {
                (Density::NAN, Density::NAN)
            };
            CohortBlock {
                cost_basis: CostBasisByPercentile {
                    by_coin: by_coin[cohort],
                    by_dollar: by_dollar[cohort],
                },
                supply_density,
                capital_density,
            }
        })
    }
}
