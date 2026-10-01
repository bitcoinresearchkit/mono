use bitview_cohort::{AmountRange, SpendableType};
use bitview_plugin_distribution_common::state::UTXOCohortState;
use brk_types::{Height, Sats, StoredU64};
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

    pub fn import(&mut self, metrics: &CohortMetrics, height: Height) -> Option<()> {
        let previous = height.decremented()?;
        for ((state, supply), count) in self
            .amount_range
            .iter_mut()
            .zip(metrics.supply.total.cohorts.utxo_amount.iter())
            .zip(metrics.outputs.unspent_count.cohorts.utxo_amount.iter())
            .chain(
                self.type_
                    .iter_mut()
                    .zip(metrics.supply.total.cohorts.type_.iter())
                    .zip(metrics.outputs.unspent_count.cohorts.type_.iter()),
            )
        {
            Self::import_one(state, &supply.sats.height, &count.height, previous)?;
        }
        Some(())
    }

    fn import_one(
        state: &mut UTXOCohortState<MinimalRealizedState, ()>,
        supply: &impl ReadableVec<Height, Sats>,
        count: &impl ReadableVec<Height, StoredU64>,
        height: Height,
    ) -> Option<()> {
        state.supply.value = supply.collect_one(height)?;
        state.supply.utxo_count = u64::from(count.collect_one(height)?);
        Some(())
    }
}
