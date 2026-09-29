use crate::state::{Transacted, UTXOStates};
use brk_types::Cents;
impl UTXOStates {
    pub fn receive_details(&mut self, received: &Transacted, price: Cents) {
        for (ty, state) in self.type_.iter_typed_mut() {
            let supply = received.by_type.get(ty);
            if supply.utxo_count > 0 {
                state.receive_utxo(supply, price);
            }
        }
        for (group, supply) in received.by_size_group.iter_typed() {
            if supply.utxo_count > 0 {
                self.amount_range.get_mut(group).receive_utxo(supply, price);
            }
        }
    }
}
