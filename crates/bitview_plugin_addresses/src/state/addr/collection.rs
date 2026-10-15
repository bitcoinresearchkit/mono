use bitview_cohort::{AddressType, AmountRange};
use brk_types::Height;

use super::AddrCohortState;
use crate::{
    addr::{AddrMetricsState, AddressVecs},
    balance::Balances,
};

pub struct AddrStates {
    pub amount_range: AmountRange<AddrCohortState>,
    /// The address types' running counters, retained between updates like the bands.
    pub metrics: AddrMetricsState,
}

impl Default for AddrStates {
    fn default() -> Self {
        Self {
            amount_range: AmountRange::from_fn(|_| AddrCohortState::new()),
            metrics: AddrMetricsState::default(),
        }
    }
}

impl AddrStates {
    /// The bands' states and the counters at the end of the block before `height`.
    pub fn restore(
        &mut self,
        balances: &Balances,
        types: &AddressType<AddressVecs>,
        height: Height,
    ) -> Option<()> {
        let previous_height = height.decremented()?;
        for (state, band) in self.amount_range.iter_mut().zip(balances.bands.iter()) {
            band.restore(state, previous_height)?;
        }
        self.metrics = AddrMetricsState::restore(types, height)?;
        Some(())
    }

    pub fn reset_block(&mut self) {
        self.amount_range
            .iter_mut()
            .for_each(|state| state.inner.reset_single_iteration_values());
    }
}
