use bitview_cohort::{AgeAggregate, AgeAggregateId};
use brk_types::{Cents, PartsPerMillion32};

use super::{density::SupplyDensity, price_distribution::PriceDistribution};
use crate::ProjectedBucket;

/// One shared projection populates every cohort's statistics and supply density.
pub(super) struct MetricBuckets {
    pub prices: AgeAggregate<PriceDistribution>,
    pub density: AgeAggregate<SupplyDensity<PartsPerMillion32>>,
}

impl Default for MetricBuckets {
    fn default() -> Self {
        Self {
            prices: AgeAggregate::default(),
            density: AgeAggregate::from_fn(|_| SupplyDensity::NAN),
        }
    }
}

impl MetricBuckets {
    pub fn update(
        &mut self,
        entries: impl Iterator<Item = ProjectedBucket<1, { AgeAggregateId::ALL.len() }>>,
        spot: Cents,
    ) {
        for prices in self.prices.iter_mut() {
            prices.clear();
        }
        let spot = spot.finite_inner().filter(|&p| p > 0).map(u128::from);
        let mut valid = spot.is_some();
        let spot = spot.unwrap_or_default();
        let lower = spot * 95;
        let upper = spot * 105;
        let mut profits = AgeAggregate::<u128>::default();
        let mut losses = AgeAggregate::<u128>::default();
        for bucket in entries {
            let price = bucket.price;
            let finite = price.finite_inner().map(u128::from);
            valid &= finite.is_some();
            let mut band = match finite {
                Some(price) if price * 100 >= lower && price * 100 <= upper => {
                    Some(if price <= spot {
                        &mut profits
                    } else {
                        &mut losses
                    })
                }
                _ => None,
            };
            for &id in AgeAggregateId::ALL {
                let [sats] = bucket.weighted[id.index()];
                id.select_mut(&mut self.prices).push(price, sats);
                if let Some(target) = band.as_deref_mut() {
                    *id.select_mut(target) += sats.as_u128();
                }
            }
        }
        self.density = AgeAggregate::from_fn(|id| {
            if valid {
                SupplyDensity::from_sums(
                    id.select(&self.prices).total_sats(),
                    *id.select(&profits),
                    *id.select(&losses),
                )
            } else {
                SupplyDensity::NAN
            }
        });
    }
}

#[cfg(test)]
#[path = "metric_buckets_tests.rs"]
mod tests;
