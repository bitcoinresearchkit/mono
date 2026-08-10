use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap},
    path::Path,
};

use brk_cohort::{
    AgeRangeId, CohortContext, Filtered, PROFITABILITY_RANGE_COUNT, TERM_NAMES, UTXO_ALL_NAME,
};
use brk_error::Result;
use brk_types::{Cents, CentsCompact, Date, Dollars, PartsPerMillion32, Sats, UrpdRaw};
use rayon::prelude::*;
use vecdb::ColumnId;

use crate::distribution::metrics::{CostBasis, ProfitabilityMetrics};

use super::{
    fenwick::{PercentileResult, ProfitabilityRangeResult},
    groups::UTXOCohorts,
};

use super::COST_BASIS_PRICE_DIGITS;

impl UTXOCohorts {
    /// Compute and push percentiles + profitability for aggregate cohorts.
    ///
    /// Percentiles and profitability are computed per-block from the Fenwick tree.
    /// Disk distributions are written only at day boundaries via K-way merge.
    pub(crate) fn push_aggregate_percentiles(
        &mut self,
        spot_price: Cents,
        date_opt: Option<Date>,
        states_path: &Path,
    ) -> Result<()> {
        if self.caches.fenwick.is_initialized() {
            self.push_fenwick_results(spot_price);
        }

        // Disk distributions only at day boundaries
        if let Some(date) = date_opt {
            self.write_disk_distributions(date, states_path)?;
        }

        Ok(())
    }

    /// Iterate over the current in-memory age-cohort URPD entries.
    ///
    /// Prices use the same rounding as the persisted daily distributions, so
    /// consumers can avoid writing and immediately rereading the current day.
    pub(crate) fn age_range_urpd_entries(
        &self,
    ) -> impl Iterator<Item = (AgeRangeId, CentsCompact, Sats)> + '_ {
        AgeRangeId::ALL.iter().copied().flat_map(move |id| {
            let cohort = id.select(&self.age_range);
            cohort.state.iter().flat_map(move |state| {
                state
                    .cost_basis_map()
                    .iter()
                    .map(move |(&price, &sats)| (id, rounded_urpd_price(price), sats))
            })
        })
    }

    /// Push all Fenwick-derived per-block results: percentiles, density, profitability.
    fn push_fenwick_results(&mut self, spot_price: Cents) {
        let (all_d, sth_d, lth_d) = self.caches.fenwick.density(spot_price);

        let all = self.caches.fenwick.percentiles_all();
        push_cost_basis(&all, all_d, &mut self.all.metrics.cost_basis);

        let sth = self.caches.fenwick.percentiles_sth();
        push_cost_basis(&sth, sth_d, &mut self.sth.metrics.cost_basis);

        let lth = self.caches.fenwick.percentiles_lth();
        push_cost_basis(&lth, lth_d, &mut self.lth.metrics.cost_basis);

        let prof = self.caches.fenwick.profitability(spot_price);
        push_profitability(&prof, &mut self.profitability);
    }

    /// K-way merge only for writing daily cost basis distributions to disk.
    fn write_disk_distributions(&mut self, date: Date, states_path: &Path) -> Result<()> {
        let sth_filter = self.sth.metrics.filter.clone();

        AgeRangeId::ALL
            .iter()
            .map(|&id| (id, id.select(&self.age_range)))
            .collect::<Vec<_>>()
            .into_par_iter()
            .try_for_each(|(id, sub)| -> Result<()> {
                let Some(state) = sub.state.as_ref() else {
                    return Ok(());
                };
                let mut merged: Vec<(CentsCompact, Sats)> = Vec::new();
                for (&price, &sats) in state.cost_basis_map().iter() {
                    let rounded = rounded_urpd_price(price);
                    if let Some(last) = merged.last_mut()
                        && last.0 == rounded
                    {
                        last.1 += sats;
                    } else {
                        merged.push((rounded, sats));
                    }
                }
                let full = CohortContext::Utxo.prefixed(id.name().id);
                UrpdRaw::write(states_path, &full, date, merged.into_iter())
            })?;

        let maps: Vec<_> = self
            .age_range
            .iter()
            .filter_map(|sub| {
                let state = sub.state.as_ref()?;
                let map = state.cost_basis_map();
                if map.is_empty() {
                    return None;
                }
                let is_sth = sth_filter.includes(sub.filter());
                Some((map, is_sth))
            })
            .collect();

        if maps.is_empty() {
            return Ok(());
        }

        let cap = maps.iter().map(|(m, _)| m.len()).max().unwrap_or(0);
        let mut targets = AllSthLth {
            all: MergeTarget::new(cap),
            sth: MergeTarget::new(cap),
            lth: MergeTarget::new(cap),
        };

        merge_k_way(&maps, &mut targets);

        [
            (UTXO_ALL_NAME.id, targets.all.merged),
            (TERM_NAMES.short.id, targets.sth.merged),
            (TERM_NAMES.long.id, targets.lth.merged),
        ]
        .into_par_iter()
        .try_for_each(|(name, merged)| {
            UrpdRaw::write(states_path, name, date, merged.into_iter())
        })?;

        Ok(())
    }
}

#[inline]
fn rounded_urpd_price(price: CentsCompact) -> CentsCompact {
    price.round_to_dollar(COST_BASIS_PRICE_DIGITS)
}

/// Push percentiles + density to cost basis vecs.
#[inline(always)]
fn push_cost_basis(
    percentiles: &PercentileResult,
    density: PartsPerMillion32,
    cost_basis: &mut CostBasis,
) {
    cost_basis.push_minmax(percentiles.min_price, percentiles.max_price);
    cost_basis.push_percentiles(&percentiles.sat_prices, &percentiles.usd_prices);
    cost_basis.push_density(density);
}

#[inline(always)]
fn raw_usd_to_dollars(raw: u128) -> Dollars {
    Dollars::from(raw as f64 / 1e10)
}

fn push_profitability(
    buckets: &[ProfitabilityRangeResult; PROFITABILITY_RANGE_COUNT],
    metrics: &mut ProfitabilityMetrics,
) {
    metrics.push_ranges(
        (*buckets).map(|range| Sats::from(range.all_sats)),
        (*buckets).map(|range| Sats::from(range.sth_sats)),
        (*buckets).map(|range| raw_usd_to_dollars(range.all_usd)),
        (*buckets).map(|range| raw_usd_to_dollars(range.sth_usd)),
    );
}

// ---------------------------------------------------------------------------
// K-way merge (retained only for disk distribution writes)
// ---------------------------------------------------------------------------

struct AllSthLth<T> {
    all: T,
    sth: T,
    lth: T,
}

impl<T> AllSthLth<T> {
    fn term_mut(&mut self, is_sth: bool) -> &mut T {
        if is_sth { &mut self.sth } else { &mut self.lth }
    }

    fn for_each_mut(&mut self, mut f: impl FnMut(&mut T)) {
        f(&mut self.all);
        f(&mut self.sth);
        f(&mut self.lth);
    }
}

/// Merge target that only collects rounded (price, sats) pairs for disk distribution.
struct MergeTarget {
    price_sats: u64,
    merged: Vec<(CentsCompact, Sats)>,
}

impl MergeTarget {
    fn new(cap: usize) -> Self {
        Self {
            price_sats: 0,
            merged: Vec::with_capacity(cap),
        }
    }

    #[inline]
    fn accumulate(&mut self, amount: u64) {
        self.price_sats += amount;
    }

    fn finalize_price(&mut self, price: CentsCompact) {
        if self.price_sats > 0 {
            let rounded = rounded_urpd_price(price);
            if let Some((lp, ls)) = self.merged.last_mut()
                && *lp == rounded
            {
                *ls += Sats::from(self.price_sats);
            } else {
                self.merged.push((rounded, Sats::from(self.price_sats)));
            }
        }
        self.price_sats = 0;
    }
}

/// K-way merge via BinaryHeap over BTreeMap iterators.
/// Only builds merged distribution for disk writes.
fn merge_k_way(
    maps: &[(&BTreeMap<CentsCompact, Sats>, bool)],
    targets: &mut AllSthLth<MergeTarget>,
) {
    let mut iters: Vec<_> = maps
        .iter()
        .map(|(map, is_sth)| (map.iter().peekable(), *is_sth))
        .collect();

    let mut heap: BinaryHeap<Reverse<(CentsCompact, usize)>> =
        BinaryHeap::with_capacity(iters.len());
    for (i, (iter, _)) in iters.iter_mut().enumerate() {
        if let Some(&(&price, _)) = iter.peek() {
            heap.push(Reverse((price, i)));
        }
    }

    let mut current_price: Option<CentsCompact> = None;

    while let Some(Reverse((price, ci))) = heap.pop() {
        let (ref mut iter, is_sth) = iters[ci];
        let (_, &sats) = iter.next().unwrap();
        let amount = u64::from(sats);

        if let Some(prev) = current_price
            && prev != price
        {
            targets.for_each_mut(|t| t.finalize_price(prev));
        }

        current_price = Some(price);
        targets.all.accumulate(amount);
        targets.term_mut(is_sth).accumulate(amount);

        if let Some(&(&next_price, _)) = iter.peek() {
            heap.push(Reverse((next_price, ci)));
        }
    }

    if let Some(price) = current_price {
        targets.for_each_mut(|t| t.finalize_price(price));
    }
}
