use bitview_cohort::{ProfitabilityRange, ProfitabilityRangeId, compute_profitability_boundaries};
use bitview_distribution::state::cost_basis::{PriceIndex, PriceTotals, age_index::AgeIndexLive};
use brk_types::{Cents, Version};

use crate::bucket::Bucket;

/// Writer-owned derived state, retained only after the complete update succeeds.
pub(crate) type LiveState = AgeIndexLive<(Version, Version, (u64, u64))>;

/// Each band's supply and capital at `spot`, and the filters' totals.
pub(crate) fn ranges(index: &PriceIndex<4>, spot: Cents) -> (ProfitabilityRange<Bucket>, Bucket) {
    let mut result = ProfitabilityRange::default();
    let total = index.totals();
    if total.sats[0] <= 0 {
        return (result, Bucket::default());
    }
    let mut previous = PriceTotals::default();
    for (i, boundary) in compute_profitability_boundaries(spot)
        .into_iter()
        .enumerate()
    {
        let current = index.before(boundary);
        *ProfitabilityRangeId::ALL[i].select_mut(&mut result) = Bucket::between(previous, current);
        previous = current;
    }
    *ProfitabilityRangeId::ALL
        .last()
        .unwrap()
        .select_mut(&mut result) = Bucket::between(previous, total);
    (result, Bucket::between(PriceTotals::default(), total))
}
