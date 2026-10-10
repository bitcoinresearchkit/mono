use std::array;

use bitview_compute::{FenwickNode, FenwickTree};
use bitview_primitives::{CentsCompact, PERCENTILES, PERCENTILES_LEN};
use bitview_urpd::{COMPUTE_VERSION, COST_BASIS_PRICE_DIGITS};
use brk_types::{Cents, Version};

use super::{PercentileResult, PriceTotals};

/// Changes with the price grid, shared with the URPD: a price index's owner adds it to the
/// version of what it computes from the index.
pub const PRICE_INDEX_VERSION: Version = COMPUTE_VERSION;

/// A derived price index. Owners choose only the filters their queries require.
#[derive(Clone)]
pub struct PriceIndex<const N: usize> {
    tree: FenwickTree<PriceTotals<N>>,
    totals: PriceTotals<N>,
}

// Bucket mapping: `COST_BASIS_PRICE_DIGITS` significant digits of the price in cents, as
// `Cents::round_to_significant` rounds. With 4 digits: exact cents under $100, then 9,000 buckets
// per decade (10 cents, $1, $10, $100) up to $1,000,000; higher prices share the last bucket.
const DIGITS: u32 = COST_BASIS_PRICE_DIGITS;
/// Buckets under 10^DIGITS cents, one per cent.
const EXACT: usize = 10usize.pow(DIGITS);
/// Buckets in each decade above `EXACT`.
const PER_DECADE: usize = 9 * 10usize.pow(DIGITS - 1);
/// The decade of $1,000,000 (10^8 cents), the overflow bucket's price.
const MAX_LOG10: u32 = 8;

/// Total number of buckets (46,001 with 4 digits).
const TREE_SIZE: usize = EXACT + (MAX_LOG10 - DIGITS) as usize * PER_DECADE + 1;

#[inline]
fn bucket_to_cents(bucket: usize) -> Cents {
    if bucket < EXACT {
        return Cents::from(bucket as u64);
    }
    if bucket >= TREE_SIZE - 1 {
        return Cents::from(10u64.pow(MAX_LOG10));
    }
    let decade = (bucket - EXACT) / PER_DECADE;
    let offset = (bucket - EXACT) % PER_DECADE;
    Cents::from((offset as u64 + 10u64.pow(DIGITS - 1)) * 10u64.pow(decade as u32 + 1))
}

#[inline]
fn cents_to_bucket(price: Cents) -> usize {
    let cents = u64::from(price.round_to_significant(DIGITS));
    match cents.checked_ilog10() {
        Some(log10) if log10 >= MAX_LOG10 => TREE_SIZE - 1,
        Some(log10) if log10 >= DIGITS => {
            let step = 10u64.pow(log10 - DIGITS + 1);
            EXACT
                + (log10 - DIGITS) as usize * PER_DECADE
                + (cents / step - 10u64.pow(DIGITS - 1)) as usize
        }
        _ => cents as usize,
    }
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
        let low = self.before(Cents::from((spot * 0.95).round() as u64));
        let mid = self.tree.prefix_sum(cents_to_bucket(price));
        let high = self
            .tree
            .prefix_sum(cents_to_bucket(Cents::from((spot * 1.05).round() as u64)));
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
