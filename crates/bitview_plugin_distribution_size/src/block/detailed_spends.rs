use crate::{block::spend_delta::SpendDelta, state::UTXOStates};
use bitview_cohort::{AmountRange, SpendableType};
use brk_types::{Cents, OutputType, Sats};

#[derive(Default)]
pub struct DetailedSpends {
    by_type: SpendableType<SpendDelta>,
    by_amount: AmountRange<SpendDelta>,
}
impl DetailedSpends {
    pub fn add(&mut self, value: Sats, ty: OutputType, previous: Cents, current: Cents) {
        if ty.is_unspendable() {
            return;
        }
        let delta = SpendDelta::new(value, previous, current);
        *self.by_type.get_mut(ty) += delta;
        *self.by_amount.get_mut(value) += delta;
    }
    pub fn apply(self, states: &mut UTXOStates) {
        for (state, delta) in states.type_.iter_mut().zip(self.by_type.iter()) {
            state.send_minimal(&delta.supply, delta.previous, delta.profit, delta.loss);
        }
        for (state, delta) in states.amount_range.iter_mut().zip(self.by_amount.iter()) {
            state.send_minimal(&delta.supply, delta.previous, delta.profit, delta.loss);
        }
    }
}

#[cfg(test)]
#[path = "detailed_spends_tests.rs"]
mod tests;
