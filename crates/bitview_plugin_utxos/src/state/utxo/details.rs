use crate::state::{Transacted, UTXOStates};
use bitview_primitives::{CostBasisSnapshot, SupplyState};
use brk_types::{Cents, OutputType};
impl UTXOStates {
    /// Removes an output that left the set without being spent: supply and realized cap drop,
    /// with no spend, volume or realized profit and loss.
    pub fn lose(&mut self, ty: OutputType, lost: &SupplyState, price: Cents) {
        let snapshot = CostBasisSnapshot::from_utxo(price, lost);
        self.type_.get_mut(ty).decrement_snapshot(&snapshot);
        self.amount_range
            .get_mut(lost.value)
            .decrement_snapshot(&snapshot);
    }

    pub fn receive_details(&mut self, received: &Transacted, price: Cents) {
        for (state, supply) in self.type_.iter_mut().zip(received.by_type.iter()) {
            if supply.utxo_count > 0 {
                state.receive_utxo(supply, price);
            }
        }
        for (group, supply) in received.by_amount.iter_typed() {
            if supply.utxo_count > 0 {
                self.amount_range.get_mut(group).receive_utxo(supply, price);
            }
        }
    }
}
