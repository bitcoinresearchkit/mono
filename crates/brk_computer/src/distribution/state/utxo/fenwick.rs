use std::collections::BTreeMap;

use brk_cohort::{
    AGE_RANGE_COUNT, AgeRangeId, Filter, PROFITABILITY_RANGE_COUNT, ProfitabilityRange,
    ProfitabilityRangeId, compute_profitability_boundaries,
};
use brk_types::{Cents, CentsCompact, PartsPerMillion32, Sats};
use vecdb::ColumnId;

use crate::{
    distribution::state::PendingDelta,
    internal::{
        PERCENTILES, PERCENTILES_LEN,
        algo::{FenwickNode, FenwickTree},
    },
};

use super::{COST_BASIS_PRICE_DIGITS, PercentileResult, ProfitabilityRangeResult};

// Tier boundaries for 5-significant-digit dollar bucketing.
// Matches the rounding used by `Cents::round_to_dollar(5)`.
const TIER0_COUNT: usize = 100_000; // $0-$99,999 exact dollars
const TIER1_COUNT: usize = 90_000; // $100,000-$999,990 step $10
const OVERFLOW: usize = 1; // $1,000,000+ clamped to last bucket

const TIER1_START: usize = TIER0_COUNT;

/// Total number of buckets.
const TREE_SIZE: usize = TIER0_COUNT + TIER1_COUNT + OVERFLOW; // 190,001

/// Fenwick tree node for combined cost basis tracking.
#[derive(Clone, Copy, Default)]
struct CostBasisNode {
    all_sats: i64,
    sth_sats: i64,
    all_usd: i128,
    sth_usd: i128,
}

impl CostBasisNode {
    #[inline(always)]
    fn new_supply(sats: i64, usd: i128, is_sth: bool) -> Self {
        Self {
            all_sats: sats,
            sth_sats: if is_sth { sats } else { 0 },
            all_usd: usd,
            sth_usd: if is_sth { usd } else { 0 },
        }
    }
}

impl FenwickNode for CostBasisNode {
    #[inline(always)]
    fn add_assign(&mut self, other: &Self) {
        self.all_sats += other.all_sats;
        self.sth_sats += other.sth_sats;
        self.all_usd += other.all_usd;
        self.sth_usd += other.sth_usd;
    }
}

/// Combined Fenwick tree for per-block accurate percentile and profitability queries.
#[derive(Clone)]
pub struct CostBasisFenwick {
    tree: FenwickTree<CostBasisNode>,
    /// Running totals (sum of all underlying frequencies).
    totals: CostBasisNode,
    /// Pre-computed: which age-range cohort index is STH?
    is_sth: [bool; AGE_RANGE_COUNT],
    initialized: bool,
}

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
fn price_to_bucket(price: CentsCompact) -> usize {
    cents_to_bucket(price.into())
}

#[inline]
fn cents_to_bucket(price: Cents) -> usize {
    dollars_to_bucket(u64::from(price.round_to_dollar(COST_BASIS_PRICE_DIGITS)) / 100)
}

impl Default for CostBasisFenwick {
    fn default() -> Self {
        Self {
            tree: FenwickTree::new(TREE_SIZE),
            totals: CostBasisNode::default(),
            is_sth: [false; AGE_RANGE_COUNT],
            initialized: false,
        }
    }
}

impl CostBasisFenwick {
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Pre-compute `is_sth` lookup from the STH filter and age-range filters.
    pub fn compute_is_sth(&mut self, sth_filter: &Filter) {
        for id in AgeRangeId::ALL {
            self.is_sth[id.index()] = sth_filter.includes(id.filter());
        }
    }

    pub fn is_sth(&self, id: AgeRangeId) -> bool {
        self.is_sth[id.index()]
    }

    /// Apply a net delta from a pending map entry.
    pub fn apply_delta(&mut self, price: CentsCompact, pending: &PendingDelta, is_sth: bool) {
        let net_sats = u64::from(pending.inc) as i64 - u64::from(pending.dec) as i64;
        if net_sats == 0 {
            return;
        }
        let bucket = price_to_bucket(price);
        let delta =
            CostBasisNode::new_supply(net_sats, price.as_u128() as i128 * net_sats as i128, is_sth);
        self.tree.add(bucket, &delta);
        self.totals.add_assign(&delta);
    }

    /// Bulk-initialize from age-range maps.
    pub fn bulk_init<'a>(
        &mut self,
        maps: impl Iterator<Item = (&'a BTreeMap<CentsCompact, Sats>, bool)>,
    ) {
        self.tree.reset();
        self.totals = CostBasisNode::default();

        for (map, is_sth) in maps {
            for (&price, &sats) in map.iter() {
                let bucket = price_to_bucket(price);
                let s = u64::from(sats) as i64;
                let node =
                    CostBasisNode::new_supply(s, price.as_u128() as i128 * s as i128, is_sth);
                self.tree.add_raw(bucket, &node);
                self.totals.add_assign(&node);
            }
        }

        self.tree.build_in_place();
        self.initialized = true;
    }

    // -----------------------------------------------------------------------
    // Percentile queries
    // -----------------------------------------------------------------------

    /// Compute sat-weighted and usd-weighted percentile prices for ALL cohort.
    pub fn percentiles_all(&self) -> PercentileResult {
        self.compute_percentiles(
            self.totals.all_sats,
            self.totals.all_usd,
            |n| n.all_sats,
            |n| n.all_usd,
        )
    }

    /// Compute percentile prices for STH cohort.
    pub fn percentiles_sth(&self) -> PercentileResult {
        self.compute_percentiles(
            self.totals.sth_sats,
            self.totals.sth_usd,
            |n| n.sth_sats,
            |n| n.sth_usd,
        )
    }

    /// Compute percentile prices for LTH cohort (all - sth per node).
    pub fn percentiles_lth(&self) -> PercentileResult {
        self.compute_percentiles(
            self.totals.all_sats - self.totals.sth_sats,
            self.totals.all_usd - self.totals.sth_usd,
            |n| n.all_sats - n.sth_sats,
            |n| n.all_usd - n.sth_usd,
        )
    }

    fn compute_percentiles(
        &self,
        total_sats: i64,
        total_usd: i128,
        sat_field: impl Fn(&CostBasisNode) -> i64,
        usd_field: impl Fn(&CostBasisNode) -> i128,
    ) -> PercentileResult {
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

        let mut sat_buckets = [0usize; PERCENTILES_LEN + 2];
        self.tree.kth(&sat_targets, &sat_field, &mut sat_buckets);

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

            let mut usd_buckets = [0usize; PERCENTILES_LEN];
            self.tree.kth(&usd_targets, &usd_field, &mut usd_buckets);

            (0..PERCENTILES_LEN).for_each(|i| {
                result.usd_prices[i] = bucket_to_cents(usd_buckets[i]);
            });
        }

        result
    }

    // -----------------------------------------------------------------------
    // Supply density queries (±5% of spot price)
    // -----------------------------------------------------------------------

    /// Compute supply density: % of supply with cost basis within ±5% of spot.
    pub fn density(
        &self,
        spot_price: Cents,
    ) -> (PartsPerMillion32, PartsPerMillion32, PartsPerMillion32) {
        if self.totals.all_sats <= 0 {
            return (
                PartsPerMillion32::ZERO,
                PartsPerMillion32::ZERO,
                PartsPerMillion32::ZERO,
            );
        }

        let range = self.density_range(spot_price);
        let all_range = range.all_sats.max(0);
        let sth_range = range.sth_sats.max(0);
        let lth_range = all_range - sth_range;

        let lth_total = self.totals.all_sats - self.totals.sth_sats;
        (
            Self::to_ppm(all_range, self.totals.all_sats),
            Self::to_ppm(sth_range, self.totals.sth_sats),
            Self::to_ppm(lth_range, lth_total),
        )
    }

    fn density_range(&self, spot_price: Cents) -> CostBasisNode {
        let spot_f64 = u64::from(spot_price) as f64;
        let low = Cents::from((spot_f64 * 0.95) as u64);
        let high = Cents::from((spot_f64 * 1.05) as u64);

        let low_bucket = cents_to_bucket(low);
        let high_bucket = cents_to_bucket(high);

        let cum_high = self.tree.prefix_sum(high_bucket);
        let cum_low = if low_bucket > 0 {
            self.tree.prefix_sum(low_bucket - 1)
        } else {
            CostBasisNode::default()
        };

        CostBasisNode {
            all_sats: cum_high.all_sats - cum_low.all_sats,
            sth_sats: cum_high.sth_sats - cum_low.sth_sats,
            all_usd: cum_high.all_usd - cum_low.all_usd,
            sth_usd: cum_high.sth_usd - cum_low.sth_usd,
        }
    }

    #[inline(always)]
    fn to_ppm(range: i64, total: i64) -> PartsPerMillion32 {
        if total <= 0 {
            PartsPerMillion32::ZERO
        } else {
            PartsPerMillion32::from(range as f64 / total as f64)
        }
    }

    // -----------------------------------------------------------------------
    // Profitability queries
    // -----------------------------------------------------------------------

    /// Compute profitability range buckets from current spot price.
    /// Returns exact STH/LTH values for every profitability range.
    pub fn profitability(&self, spot_price: Cents) -> ProfitabilityRange<ProfitabilityRangeResult> {
        let mut result = ProfitabilityRange::default();

        if self.totals.all_sats <= 0 {
            return result;
        }

        let boundaries = compute_profitability_boundaries(spot_price);

        let mut prev = CostBasisNode::default();

        for (i, &boundary) in boundaries.iter().enumerate() {
            let boundary_bucket = cents_to_bucket(boundary);
            let cum = if boundary_bucket > 0 {
                self.tree.prefix_sum(boundary_bucket - 1)
            } else {
                CostBasisNode::default()
            };
            *ProfitabilityRangeId::ALL[i].get_mut(&mut result) =
                ProfitabilityRangeResult::from_all_and_sth(
                    (cum.all_sats - prev.all_sats).max(0) as u64,
                    (cum.all_usd - prev.all_usd).max(0) as u128,
                    (cum.sth_sats - prev.sth_sats).max(0) as u64,
                    (cum.sth_usd - prev.sth_usd).max(0) as u128,
                );
            prev = cum;
        }

        // Last range: everything >= last boundary
        *ProfitabilityRangeId::ALL[PROFITABILITY_RANGE_COUNT - 1].get_mut(&mut result) =
            ProfitabilityRangeResult::from_all_and_sth(
                (self.totals.all_sats - prev.all_sats).max(0) as u64,
                (self.totals.all_usd - prev.all_usd).max(0) as u128,
                (self.totals.sth_sats - prev.sth_sats).max(0) as u64,
                (self.totals.sth_usd - prev.sth_usd).max(0) as u128,
            );

        result
    }
}
