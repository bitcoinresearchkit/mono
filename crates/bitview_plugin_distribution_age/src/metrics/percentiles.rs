use bitview_cohort::{ByTerm, ProfitabilityRange, UTXOAggregate};
use brk_types::Cents;

use crate::{
    metrics::{CohortMetrics, CostBasisBlockData},
    state::UTXOStates,
};

impl CohortMetrics {
    pub fn push_aggregate_percentiles(&mut self, states: &UTXOStates, spot_price: Cents) {
        if states.fenwick().is_initialized() {
            self.push_fenwick_results(states, spot_price);
        }
    }

    fn push_fenwick_results(&mut self, states: &UTXOStates, spot_price: Cents) {
        let fenwick = states.fenwick();
        let (all_density, sth_density, lth_density) = fenwick.density(spot_price);
        self.cost_basis.push(UTXOAggregate {
            all: CostBasisBlockData::from_percentiles(fenwick.percentiles_all(), all_density),
            sth: CostBasisBlockData::from_percentiles(fenwick.percentiles_sth(), sth_density),
            lth: CostBasisBlockData::from_percentiles(fenwick.percentiles_lth(), lth_density),
        });

        let profitability = fenwick.profitability(spot_price);
        self.profitability.push(
            spot_price,
            ByTerm {
                short: ProfitabilityRange::from_fn(|id| id.select(&profitability).supply.short),
                long: ProfitabilityRange::from_fn(|id| id.select(&profitability).supply.long),
            },
            ByTerm {
                short: ProfitabilityRange::from_fn(|id| {
                    id.select(&profitability).realized_cap.short
                }),
                long: ProfitabilityRange::from_fn(|id| id.select(&profitability).realized_cap.long),
            },
        );
    }
}
