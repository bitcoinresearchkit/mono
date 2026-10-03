use bitview_primitives::{CentsCompact, CostBasisByPercentile, PERCENTILES, PERCENTILES_LEN};
use brk_types::{Cents, Sats};

use super::price_stats::PriceStats;

struct PricePrefix {
    price: CentsCompact,
    sats: u128,
    value: u128,
}

/// Reusable cumulative buckets for direct percentile lookup after one projection pass.
pub(super) struct PriceDistribution {
    entries: Vec<PricePrefix>,
    total_sats: u128,
    total_value: u128,
    second_moment: Option<u128>,
}

impl Default for PriceDistribution {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            total_sats: 0,
            total_value: 0,
            second_moment: Some(0),
        }
    }
}

impl PriceDistribution {
    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_sats = 0;
        self.total_value = 0;
        self.second_moment = Some(0);
    }

    pub fn push(&mut self, price: CentsCompact, sats: Sats) {
        if sats == Sats::ZERO {
            return;
        }
        let value = price.as_u128() * sats.as_u128();
        self.total_sats += sats.as_u128();
        self.total_value += value;
        // A u32 price squared times u64 sats fits u128; only the sum can overflow.
        self.second_moment = self
            .second_moment
            .and_then(|sum| sum.checked_add(value * price.as_u128()));
        self.entries.push(PricePrefix {
            price,
            sats: self.total_sats,
            value: self.total_value,
        });
    }

    pub fn total_sats(&self) -> u128 {
        self.total_sats
    }

    pub fn stats(&self) -> PriceStats {
        PriceStats {
            cost_basis: CostBasisByPercentile {
                per_coin: self.percentiles(self.total_sats, |entry| entry.sats),
                per_dollar: self.percentiles(self.total_value, |entry| entry.value),
            },
            capitalized_price: self
                .second_moment
                .and_then(|value| value.checked_div(self.total_value))
                .map(Cents::from)
                .unwrap_or(Cents::NAN),
        }
    }

    fn percentiles(
        &self,
        total: u128,
        cumulative: impl Fn(&PricePrefix) -> u128,
    ) -> [Cents; PERCENTILES_LEN] {
        if total == 0 {
            return [Cents::default(); PERCENTILES_LEN];
        }
        PERCENTILES.map(|percentile| {
            let target = (total * u128::from(percentile) / 100).saturating_sub(1);
            let index = self
                .entries
                .partition_point(|entry| cumulative(entry) <= target);
            self.entries[index].price.into()
        })
    }
}
