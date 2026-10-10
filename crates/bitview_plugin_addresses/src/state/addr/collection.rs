use bitview_cohort::AmountRange;
use brk_types::Height;

use super::AddrCohortState;
use crate::balance::Balances;

pub struct AddrStates {
    pub amount_range: AmountRange<AddrCohortState>,
}

impl AddrStates {
    pub fn new() -> Self {
        Self {
            amount_range: AmountRange::new(|_| AddrCohortState::new()),
        }
    }

    /// The bands' states at the end of the block before `height`.
    pub fn restore(&mut self, balances: &Balances, height: Height) -> Option<()> {
        let previous_height = height.decremented()?;
        for (state, band) in self.amount_range.iter_mut().zip(balances.bands.iter()) {
            band.restore(state, previous_height)?;
        }
        Some(())
    }

    pub fn reset_block(&mut self) {
        self.amount_range
            .iter_mut()
            .for_each(|state| state.inner.reset_single_iteration_values());
    }
}
