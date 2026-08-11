use std::path::Path;

use brk_cohort::{ByTerm, Filter, ProfitabilityRangeId, Term, UTXOAggregate};
use brk_error::Result;
use brk_types::{Cents, Date};
use vecdb::ColumnId;

use crate::distribution::{
    metrics::{CohortMetrics, CostBasisBlockData},
    state::UTXOStates,
};

impl CohortMetrics {
    pub fn push_aggregate_percentiles(
        &mut self,
        states: &UTXOStates,
        spot_price: Cents,
        date: Option<Date>,
        states_path: &Path,
    ) -> Result<()> {
        if states.fenwick().is_initialized() {
            self.push_fenwick_results(states, spot_price);
        }
        if let Some(date) = date {
            states.write_urpds(date, states_path, &Filter::Term(Term::Sth))?;
        }
        Ok(())
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
                short: ProfitabilityRangeId::map_ref(&profitability, |row| row.supply.short),
                long: ProfitabilityRangeId::map_ref(&profitability, |row| row.supply.long),
            },
            ByTerm {
                short: ProfitabilityRangeId::map_ref(&profitability, |row| row.realized_cap.short),
                long: ProfitabilityRangeId::map_ref(&profitability, |row| row.realized_cap.long),
            },
        );
    }
}
