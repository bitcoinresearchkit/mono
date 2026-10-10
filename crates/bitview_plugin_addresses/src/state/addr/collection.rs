use bitview_cohort::AmountRange;
use brk_types::{Cents, Height};

use super::AddrCohortState;
use crate::balance::BalanceVecs;

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
    pub fn restore(&mut self, balances: &AmountRange<BalanceVecs>, height: Height) -> Option<()> {
        let previous_height = height.decremented()?;
        for (state, band) in self.amount_range.iter_mut().zip(balances.iter()) {
            band.restore(state, previous_height)?;
        }
        Some(())
    }

    pub fn push(&self, balances: &mut AmountRange<BalanceVecs>, height_price: Cents) {
        for (band, state) in balances.iter_mut().zip(self.amount_range.iter()) {
            band.push(state, height_price);
        }
    }

    pub fn reset_block(&mut self) {
        self.amount_range
            .iter_mut()
            .for_each(|state| state.inner.reset_single_iteration_values());
    }
}
