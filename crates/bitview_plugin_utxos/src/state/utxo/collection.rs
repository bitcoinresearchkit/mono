use bitview_cohort::{AmountRange, SpendableType};
use bitview_distribution::state::CohortState;
use brk_types::Height;
use vecdb::ReadableVec;

use crate::{metrics::CohortMetrics, state::MinimalRealizedState};

#[derive(Default)]
pub struct UTXOStates {
    pub amount_range: AmountRange<CohortState<MinimalRealizedState, ()>>,
    pub type_: SpendableType<CohortState<MinimalRealizedState, ()>>,
}

impl UTXOStates {
    pub fn restore(&mut self, metrics: &CohortMetrics, height: Height) -> Option<()> {
        let previous = height.decremented()?;
        for (state, cohort) in self
            .amount_range
            .iter_mut()
            .zip(metrics.amounts.iter())
            .chain(self.type_.iter_mut().zip(metrics.types.iter()))
        {
            state.supply.value = cohort.supply_at(previous)?;
            state.supply.utxo_count = u64::from(cohort.utxos.count.stored.collect_one(previous)?);
        }
        Some(())
    }
}
