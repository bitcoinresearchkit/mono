use bitview_cohort::{AmountRange, SpendableType};
use bitview_distribution::state::UTXOCohortState;
use bitview_primitives::Count;
use brk_types::{Height, Sats};
use vecdb::ReadableVec;

use crate::{metrics::CohortMetrics, state::MinimalRealizedState};

pub struct UTXOStates {
    pub amount_range: AmountRange<UTXOCohortState<MinimalRealizedState, ()>>,
    pub type_: SpendableType<UTXOCohortState<MinimalRealizedState, ()>>,
}
impl UTXOStates {
    pub fn new() -> Self {
        Self {
            amount_range: AmountRange::new(|_| UTXOCohortState::new(())),
            type_: SpendableType::new(|_| UTXOCohortState::new(())),
        }
    }

    pub fn restore(&mut self, metrics: &CohortMetrics, height: Height) -> Option<()> {
        let previous = height.decremented()?;
        for (state, cohort) in self
            .amount_range
            .iter_mut()
            .zip(metrics.amounts.iter())
            .chain(
                self.type_
                    .iter_mut()
                    .zip(metrics.types.iter().map(|types| &types.cohort)),
            )
        {
            Self::restore_one(
                state,
                &cohort.supply.total.stored,
                &cohort.outputs.unspent_count.stored,
                previous,
            )?;
        }
        Some(())
    }

    fn restore_one(
        state: &mut UTXOCohortState<MinimalRealizedState, ()>,
        supply: &impl ReadableVec<Height, Sats>,
        count: &impl ReadableVec<Height, Count>,
        height: Height,
    ) -> Option<()> {
        state.supply.value = supply.collect_one(height)?;
        state.supply.utxo_count = u64::from(count.collect_one(height)?);
        Some(())
    }
}
