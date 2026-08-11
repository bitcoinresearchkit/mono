use std::ops::AddAssign;

use brk_types::{Cents, Sats};

#[derive(Debug, Default, Clone)]
pub struct UnrealizedState {
    pub supply_in_profit: Sats,
    pub supply_in_loss: Sats,
    pub unrealized_profit: Cents,
    pub unrealized_loss: Cents,
    pub capitalized_cap_in_profit_raw: u128,
    pub capitalized_cap_in_loss_raw: u128,
}

impl UnrealizedState {
    pub const ZERO: Self = Self {
        supply_in_profit: Sats::ZERO,
        supply_in_loss: Sats::ZERO,
        unrealized_profit: Cents::ZERO,
        unrealized_loss: Cents::ZERO,
        capitalized_cap_in_profit_raw: 0,
        capitalized_cap_in_loss_raw: 0,
    };
}

impl AddAssign<&Self> for UnrealizedState {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &Self) {
        self.supply_in_profit += rhs.supply_in_profit;
        self.supply_in_loss += rhs.supply_in_loss;
        self.unrealized_profit += rhs.unrealized_profit;
        self.unrealized_loss += rhs.unrealized_loss;
        self.capitalized_cap_in_profit_raw += rhs.capitalized_cap_in_profit_raw;
        self.capitalized_cap_in_loss_raw += rhs.capitalized_cap_in_loss_raw;
    }
}
