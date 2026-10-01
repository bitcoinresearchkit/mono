use bitview_cohort::{AmountRange, SpendableType};
use brk_types::{OutputType, Sats, SupplyState};
use vecdb::unlikely;

#[derive(Default, Debug)]
pub struct Transacted {
    pub by_type: SpendableType<SupplyState>,
    pub by_amount: AmountRange<SupplyState>,
}

impl Transacted {
    pub fn iterate(&mut self, value: Sats, ty: OutputType) {
        if unlikely(ty.is_unspendable()) {
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
