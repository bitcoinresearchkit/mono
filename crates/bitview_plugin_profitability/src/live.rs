use bitview_cohort::{
    BREAK_EVEN_DIVISOR, PROFITABILITY_EDGE_DIVISORS, ProfitabilityRange, ProfitabilityRangeId,
};
use bitview_distribution::state::cost_basis::{
    GridCut, PriceIndex, PriceTotals, age_index::AgeIndexLive,
};
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
    let mut cut = GridCut::default();
    for (i, divisor) in PROFITABILITY_EDGE_DIVISORS.into_iter().enumerate() {
        // Break-even counts the spot's own bucket as profit, like the holders' densities. Cuts never
        // move back: at a zero spot every edge is zero.
        cut = cut.max(if divisor == BREAK_EVEN_DIVISOR {
            GridCut::through(spot)
        } else {
            GridCut::below(spot.as_u128() * 100, u128::from(divisor))
        });
        let current = index.before_cut(cut);
        *ProfitabilityRangeId::ALL[i].select_mut(&mut result) = Bucket::between(previous, current);
        previous = current;
    }
    *ProfitabilityRangeId::ALL
        .last()
        .unwrap()
        .select_mut(&mut result) = Bucket::between(previous, total);
    (result, Bucket::between(PriceTotals::default(), total))
}
