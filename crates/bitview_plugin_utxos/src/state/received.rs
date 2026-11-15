use bitview_cohort::{AmountRange, SpendableType};
use bitview_primitives::{Boolean, SupplyState};
use brk_types::{OutputType, Sats};
use vecdb::unlikely;

#[derive(Default, Debug)]
pub struct ReceivedOutputs {
    pub by_type: SpendableType<SupplyState>,
    pub by_amount: AmountRange<SupplyState>,
}

impl ReceivedOutputs {
    /// An output the block created; an unspendable one (Core's coin-pruning rule) never enters
    /// the UTXO set.
    pub fn add(&mut self, value: Sats, ty: OutputType, unspendable: Boolean) {
        if unlikely(unspendable.is_true()) {
            return;
        }
        let supply = SupplyState {
            utxo_count: 1,
            value,
        };
        *self.by_type.get_mut(ty) += &supply;
        *self.by_amount.get_mut(value) += &supply;
    }
}
