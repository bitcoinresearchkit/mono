use std::ops::AddAssign;

use brk_types::{Cents, Sats};

#[derive(Debug, Default, Clone)]
pub struct UnrealizedState {
    pub supply_in_profit: Sats,
    pub supply_in_loss: Sats,
    pub unrealized_profit: Cents,
    pub unrealized_loss: Cents,
}

impl UnrealizedState {
    pub(crate) const ZERO: Self = Self {
        supply_in_profit: Sats::ZERO,
        supply_in_loss: Sats::ZERO,
        unrealized_profit: Cents::ZERO,
        unrealized_loss: Cents::ZERO,
    };
}

impl AddAssign<&Self> for UnrealizedState {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &Self) {
        self.supply_in_profit += rhs.supply_in_profit;
        self.supply_in_loss += rhs.supply_in_loss;
        self.unrealized_profit += rhs.unrealized_profit;
        self.unrealized_loss += rhs.unrealized_loss;
    }
}
