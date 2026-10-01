use bitview_cohort::{ProfitabilityRange, ProfitabilityRangeId, compute_profitability_boundaries};
use bitview_plugin_distribution_common::state::cost_basis::{PriceIndex, PriceTotals, age_index};
use brk_types::{Cents, CentsCompact, Timestamp, Version};
use statedb::State;

use crate::bucket::Bucket;

/// Writer-owned derived state, retained only after the complete update succeeds.
pub(crate) struct LiveState {
    pub(crate) origins: State,
    pub(crate) index: PriceIndex<4>,
    pub(crate) prices: Vec<CentsCompact>,
    pub(crate) timestamps: Vec<Timestamp>,
    pub(crate) crossings: [usize; 3],
    pub(crate) version: (Version, Version, (u64, u64)),
}

impl LiveState {
    pub(crate) fn restore(&mut self) {
        age_index::restore(
            &mut self.index,
            &self.origins,
            &self.prices,
            &self.timestamps,
        );
    }
}
pub(crate) use bitview_plugin_distribution_common::state::cost_basis::age_index::advance;

pub(crate) fn ranges(index: &PriceIndex<4>, spot: Cents) -> ProfitabilityRange<Bucket> {
    let mut result = ProfitabilityRange::default();
    let total = index.totals();
    if total.sats[0] <= 0 {
        return result;
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
    result
}
