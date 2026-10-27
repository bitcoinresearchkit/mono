use std::array;

use bitview_compute::FenwickTree;
use bitview_primitives::{CentsCompact, PERCENTILES};
use bitview_urpd::{COST_BASIS_PRICE_DIGITS, DensityBand};
use brk_types::{Cents, Version};

use super::{PercentileResult, PriceTotals};

/// Bump when the URPD price grid (`bitview_urpd::COMPUTE_VERSION`) or the index's price ceiling
/// changes. A price index's owner adds it to the version of what it computes from the index.
pub const PRICE_INDEX_VERSION: Version = Version::new(3);

/// A derived price index. Owners choose only the filters their queries require.
#[derive(Clone)]
pub struct PriceIndex<const N: usize> {
    tree: FenwickTree<PriceTotals<N>>,
    totals: PriceTotals<N>,
}

// Bucket mapping: `COST_BASIS_PRICE_DIGITS` significant digits of the price in cents, as
// `Cents::round_to_significant` rounds. With 4 digits: exact cents under $100, then 9,000 buckets
// per decade (steps of 10 cents, $1, $10, $100, $1,000, $10,000) up to $100,000,000; higher prices
// share the last bucket.
const DIGITS: u32 = COST_BASIS_PRICE_DIGITS;
/// Buckets under 10^DIGITS cents, one per cent.
const EXACT: usize = 10usize.pow(DIGITS);
/// Buckets in each decade above `EXACT`.
const PER_DECADE: usize = 9 * 10usize.pow(DIGITS - 1);
/// The decade of $100,000,000 (10^10 cents), the overflow bucket's price: decades of headroom.
const MAX_LOG10: u32 = 10;

/// Total number of buckets (64,001 with 4 digits).
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

/// A position on the price grid: the buckets before it are below the cut.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridCut(usize);

impl GridCut {
    /// Below the exact price `numer / denom` cents: an edge between grid prices rounds up to
    /// the next one.
    pub fn below(numer: u128, denom: u128) -> Self {
        let past = |bucket: usize| u64::from(bucket_to_cents(bucket)) as u128 * denom >= numer;
        let (mut low, mut high) = (0, TREE_SIZE);
        while low < high {
            let mid = (low + high) / 2;
            if past(mid) {
                high = mid;
            } else {
                low = mid + 1;
            }
        }
        Self(low)
    }

    /// Through `price`'s bucket: prices rounding to it count, like the in-profit side of
    /// `PriceIndex::density_split`.
    pub fn through(price: Cents) -> Self {
        Self(cents_to_bucket(price) + 1)
    }
}

/// A price with its bucket, mapped once: an origin height's price is added and removed many times.
#[derive(Clone, Copy)]
pub struct BucketedPrice {
    price: CentsCompact,
    bucket: u32,
}

impl BucketedPrice {
    /// The price itself.
    #[inline]
    pub fn price(self) -> Cents {
        self.price.into()
    }
}

impl From<CentsCompact> for BucketedPrice {
    #[inline]
    fn from(price: CentsCompact) -> Self {
        Self {
            price,
            bucket: cents_to_bucket(price.into()) as u32,
        }
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

    pub(crate) fn add_raw(&mut self, price: BucketedPrice, sats: i64, filters: [bool; N]) {
        let delta = Self::delta(price.price, sats, filters);
        self.tree.add_raw(price.bucket as usize, &delta);
        self.totals += &delta;
    }

    pub(crate) fn build(&mut self) {
        self.tree.build_in_place();
    }

    #[inline]
    pub(crate) fn add(&mut self, price: BucketedPrice, sats: i64, filters: [bool; N]) {
        if sats == 0 {
            return;
        }
        let delta = Self::delta(price.price, sats, filters);
        self.tree.add(price.bucket as usize, &delta);
        self.totals += &delta;
    }

    pub fn totals(&self) -> PriceTotals<N> {
        self.totals
    }

    /// Totals of the buckets before `cut`.
    pub fn before_cut(&self, cut: GridCut) -> PriceTotals<N> {
        if cut.0 == 0 {
            PriceTotals::default()
        } else {
            self.tree.prefix_sum(cut.0 - 1)
        }
    }

    /// Totals within the URPD's density band, split at its spot: the buckets through the spot
    /// bucket (in profit), then the ones above it (in loss).
    pub fn density_split(&self, band: DensityBand) -> (PriceTotals<N>, PriceTotals<N>) {
        let cut = |(numer, denom)| self.before_cut(GridCut::below(numer, denom));
        let low = cut(band.first_inside());
        let mid = self.before_cut(GridCut::through(band.spot()));
        let high = cut(band.first_above());
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
        // Each percentile: nearest rank, the ceil(p% of total)-th unit.
        let sat_targets = totals.map(|(sats, _)| {
            (sats > 0).then(|| PERCENTILES.map(|p| (sats * i64::from(p) + 99) / 100 - 1))
        });
        let usd_targets = totals.map(|(sats, cap)| {
            (sats > 0 && cap > 0).then(|| PERCENTILES.map(|p| (cap * i128::from(p) + 99) / 100 - 1))
        });
        // Without supply (or capital) the prices are undefined.
        let sat_defined = sat_targets.map(|targets| targets.is_some());
        let usd_defined = usd_targets.map(|targets| targets.is_some());
        let sat_buckets = self.tree.kth_many(sat_targets, &|q, n| fields(q, n).0);
        let usd_buckets = self.tree.kth_many(usd_targets, &|q, n| fields(q, n).1);
        let price = |defined: bool, bucket| {
            if defined {
                bucket_to_cents(bucket)
            } else {
                Cents::NAN
            }
        };
        array::from_fn(|q| PercentileResult {
            sat_prices: array::from_fn(|i| price(sat_defined[q], sat_buckets[q][i])),
            usd_prices: array::from_fn(|i| price(usd_defined[q], usd_buckets[q][i])),
        })
    }
}
