use std::array;

use bitview_compute::{FenwickNode, FenwickTree};
use bitview_primitives::{CentsCompact, PERCENTILES, PERCENTILES_LEN};
use bitview_urpd::COST_BASIS_PRICE_DIGITS;
use brk_types::Cents;

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
    pub(crate) fn reset(&mut self) {
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

    pub(crate) fn add_raw(&mut self, price: CentsCompact, sats: i64, filters: [bool; N]) {
        let delta = Self::delta(price, sats, filters);
        self.tree.add_raw(cents_to_bucket(price.into()), &delta);
        self.totals.add_assign(&delta);
    }

    pub(crate) fn build(&mut self) {
        self.tree.build_in_place();
    }

    #[inline]
    pub(crate) fn add(&mut self, price: CentsCompact, sats: i64, filters: [bool; N]) {
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

    /// Totals within the inclusive +-5 percent density interval, split at `price`: the buckets
    /// through the spot bucket (in profit), then the ones above it (in loss).
    pub fn density_split(&self, price: Cents) -> (PriceTotals<N>, PriceTotals<N>) {
        let spot = u64::from(price) as f64;
        let low = self.before(Cents::from((spot * 0.95) as u64));
        let mid = self.tree.prefix_sum(cents_to_bucket(price));
        let high = self
            .tree
            .prefix_sum(cents_to_bucket(Cents::from((spot * 1.05) as u64)));
        let between = |from: &PriceTotals<N>, to: &PriceTotals<N>| PriceTotals {
            sats: array::from_fn(|i| to.sats[i] - from.sats[i]),
            cap: array::from_fn(|i| to.cap[i] - from.cap[i]),
        };
        (between(&low, &mid), between(&mid, &high))
    }

    pub fn percentiles<const QUERIES: usize>(
        &self,
        fields: impl Fn(usize, &PriceTotals<N>) -> (i64, i128),
    ) -> [PercentileResult; QUERIES] {
        let totals: [(i64, i128); QUERIES] = array::from_fn(|q| fields(q, &self.totals));
        let sat_targets = totals.map(|(sats, _)| {
            (sats > 0).then(|| {
                // Min, each percentile, then max.
                let mut targets = [0; PERCENTILES_LEN + 2];
                for (i, &p) in PERCENTILES.iter().enumerate() {
                    targets[i + 1] = (sats * i64::from(p) / 100 - 1).max(0);
                }
                targets[PERCENTILES_LEN + 1] = sats - 1;
                targets
            })
        });
        let usd_targets = totals.map(|(sats, cap)| {
            (sats > 0 && cap > 0)
                .then(|| PERCENTILES.map(|p| (cap * i128::from(p) / 100 - 1).max(0)))
        });
        let sat_buckets = self.tree.kth_many(sat_targets, &|q, n| fields(q, n).0);
        let usd_buckets = self.tree.kth_many(usd_targets, &|q, n| fields(q, n).1);
        array::from_fn(|q| PercentileResult {
            min_price: bucket_to_cents(sat_buckets[q][0]),
            max_price: bucket_to_cents(sat_buckets[q][PERCENTILES_LEN + 1]),
            sat_prices: array::from_fn(|i| bucket_to_cents(sat_buckets[q][i + 1])),
            usd_prices: array::from_fn(|i| bucket_to_cents(usd_buckets[q][i])),
        })
    }
}
