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
    /// The cohorts whose exact realized caps are checkpointed, in their on-disk slot order.
    pub fn capped(&self) -> impl Iterator<Item = &CohortState<MinimalRealizedState, ()>> {
        self.amount_range.iter().chain(self.type_.iter())
    }

    /// [`Self::capped`], for restoring them.
    pub fn capped_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut CohortState<MinimalRealizedState, ()>> {
        self.amount_range.iter_mut().chain(self.type_.iter_mut())
    }

    pub fn restore(&mut self, metrics: &CohortMetrics, height: Height) -> Option<()> {
        let previous = height.decremented()?;
        let (zero_state, band_states) = self.amount_range.split_mut();
        let (zero, bands) = metrics.amounts.split();
        zero.restore(zero_state, previous)?;
        for (state, cohort) in band_states
            .into_iter()
            .zip(bands)
            .chain(self.type_.iter_mut().zip(metrics.types.iter()))
        {
            state.supply.value = cohort.supply.total.stored.collect_one(previous)?;
            state.supply.utxo_count = u64::from(cohort.utxos.count.stored.collect_one(previous)?);
        }
        Some(())
    }
}
