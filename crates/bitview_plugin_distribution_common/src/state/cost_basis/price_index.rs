use std::array;

use bitview_compute::{FenwickNode, FenwickTree};
use bitview_urpd::COST_BASIS_PRICE_DIGITS;
use brk_types::{Cents, CentsCompact, PERCENTILES, PERCENTILES_LEN};

use super::{PercentileResult, PriceTotals};

/// A derived price index. Owners choose only the filters their queries require.
#[derive(Clone)]
pub struct PriceIndex<const N: usize> {
    tree: FenwickTree<PriceTotals<N>>,
    totals: PriceTotals<N>,
}

// Tier boundaries for 5-significant-digit dollar bucketing.
// Matches the rounding used by `Cents::round_to_dollar(5)`.
const TIER0_COUNT: usize = 100_000; // $0-$99,999 exact dollars
const TIER1_COUNT: usize = 90_000; // $100,000-$999,990 step $10
const OVERFLOW: usize = 1; // $1,000,000+ clamped to last bucket

const TIER1_START: usize = TIER0_COUNT;

/// Total number of buckets.
const TREE_SIZE: usize = TIER0_COUNT + TIER1_COUNT + OVERFLOW; // 190,001

// ---------------------------------------------------------------------------
// Bucket mapping: 5-significant-digit dollar precision
// Uses Cents::round_to_dollar(5) for rounding, then maps rounded dollars
// to a flat bucket index across two tiers.
// ---------------------------------------------------------------------------

/// Prices >= $1M are clamped to the last bucket.
#[inline]
fn dollars_to_bucket(dollars: u64) -> usize {
    if dollars < 100_000 {
        dollars as usize
    } else if dollars < 1_000_000 {
        TIER1_START + ((dollars - 100_000) / 10) as usize
    } else {
        TREE_SIZE - 1 // overflow bucket for $1M+
    }
}

#[inline]
fn bucket_to_cents(bucket: usize) -> Cents {
    let dollars: u64 = if bucket < TIER1_START {
        bucket as u64
    } else if bucket < TREE_SIZE - 1 {
        100_000 + (bucket - TIER1_START) as u64 * 10
    } else {
        1_000_000
    };
    Cents::from(dollars * 100)
}

#[inline]
fn cents_to_bucket(price: Cents) -> usize {
    dollars_to_bucket(u64::from(price.round_to_dollar(COST_BASIS_PRICE_DIGITS)) / 100)
}

impl<const N: usize> Default for PriceIndex<N> {
    fn default() -> Self {
        Self {
            tree: FenwickTree::new(TREE_SIZE),
            totals: PriceTotals::default(),
        }
    }
}

impl<const N: usize> PriceIndex<N> {
    pub fn reset(&mut self) {
        self.tree.reset();
        self.totals = PriceTotals::default();
    }

    fn delta(price: CentsCompact, sats: i64, filters: [bool; N]) -> PriceTotals<N> {
        let cap = price.as_u128() as i128 * sats as i128;
        PriceTotals {
            sats: filters.map(|included| if included { sats } else { 0 }),
            cap: filters.map(|included| if included { cap } else { 0 }),
        }
    }

    pub fn add_raw(&mut self, price: CentsCompact, sats: i64, filters: [bool; N]) {
        let delta = Self::delta(price, sats, filters);
        self.tree.add_raw(cents_to_bucket(price.into()), &delta);
        self.totals.add_assign(&delta);
    }

    pub fn build(&mut self) {
        self.tree.build_in_place();
    }

    #[inline]
    pub fn add(&mut self, price: CentsCompact, sats: i64, filters: [bool; N]) {
        if sats == 0 {
            return;
        }
        let delta = Self::delta(price, sats, filters);
        self.tree.add(cents_to_bucket(price.into()), &delta);
        self.totals.add_assign(&delta);
    }

    pub fn totals(&self) -> PriceTotals<N> {
        self.totals
    }

    /// Totals strictly below the represented boundary bucket.
    pub fn before(&self, price: Cents) -> PriceTotals<N> {
        let bucket = cents_to_bucket(price);
        if bucket == 0 {
            PriceTotals::default()
        } else {
            self.tree.prefix_sum(bucket - 1)
        }
    }

    /// Totals within the existing inclusive +-5 percent density interval.
    pub fn density_range(&self, price: Cents) -> PriceTotals<N> {
        let price = u64::from(price) as f64;
        let low = self.before(Cents::from((price * 0.95) as u64));
        let high = self
            .tree
            .prefix_sum(cents_to_bucket(Cents::from((price * 1.05) as u64)));
        PriceTotals {
            sats: array::from_fn(|i| high.sats[i] - low.sats[i]),
            cap: array::from_fn(|i| high.cap[i] - low.cap[i]),
        }
    }

    pub fn percentiles(
        &self,
        sat_field: impl Fn(&PriceTotals<N>) -> i64,
        usd_field: impl Fn(&PriceTotals<N>) -> i128,
    ) -> PercentileResult {
        let total_sats = sat_field(&self.totals);
        let total_usd = usd_field(&self.totals);
        let mut result = PercentileResult::default();

        if total_sats <= 0 {
            return result;
        }

        // Build sorted sat targets: [min=0, percentiles..., max=total-1]
        let mut sat_targets = [0i64; PERCENTILES_LEN + 2];
        sat_targets[0] = 0; // min
        for (i, &p) in PERCENTILES.iter().enumerate() {
            sat_targets[i + 1] = (total_sats * i64::from(p) / 100 - 1).max(0);
        }
        sat_targets[PERCENTILES_LEN + 1] = total_sats - 1; // max

        let sat_buckets = self.tree.kth(sat_targets, &sat_field);

        result.min_price = bucket_to_cents(sat_buckets[0]);
        (0..PERCENTILES_LEN).for_each(|i| {
            result.sat_prices[i] = bucket_to_cents(sat_buckets[i + 1]);
        });
        result.max_price = bucket_to_cents(sat_buckets[PERCENTILES_LEN + 1]);

        // USD-weighted percentiles (batch)
        if total_usd > 0 {
            let mut usd_targets = [0i128; PERCENTILES_LEN];
            for (i, &p) in PERCENTILES.iter().enumerate() {
                usd_targets[i] = (total_usd * i128::from(p) / 100 - 1).max(0);
            }

            let usd_buckets = self.tree.kth(usd_targets, &usd_field);

            (0..PERCENTILES_LEN).for_each(|i| {
                result.usd_prices[i] = bucket_to_cents(usd_buckets[i]);
            });
        }

        result
    }
}

#[cfg(test)]
#[path = "price_index_tests.rs"]
mod tests;
