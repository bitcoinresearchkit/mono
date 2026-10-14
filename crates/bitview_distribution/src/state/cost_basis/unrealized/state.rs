use std::ops::AddAssign;

use brk_types::{Cents, Sats};

#[derive(Debug, Default, Clone)]
pub(crate) struct UnrealizedState {
    pub(crate) supply_in_profit: Sats,
    pub(crate) supply_in_loss: Sats,
    pub(crate) unrealized_profit: Cents,
    pub(crate) unrealized_loss: Cents,
    /// Creation-date value of the supply in profit, rounded once from the exact sum.
    pub(crate) capital_in_profit: Cents,
    /// Creation-date value of the supply in loss, rounded once from the exact sum.
    pub(crate) capital_in_loss: Cents,
}

impl UnrealizedState {
    pub(crate) const ZERO: Self = Self {
        supply_in_profit: Sats::ZERO,
        supply_in_loss: Sats::ZERO,
        unrealized_profit: Cents::ZERO,
        unrealized_loss: Cents::ZERO,
        capital_in_profit: Cents::ZERO,
        capital_in_loss: Cents::ZERO,
    };
}

impl AddAssign<&Self> for UnrealizedState {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &Self) {
        self.supply_in_profit += rhs.supply_in_profit;
        self.supply_in_loss += rhs.supply_in_loss;
        self.unrealized_profit += rhs.unrealized_profit;
        self.unrealized_loss += rhs.unrealized_loss;
        self.capital_in_profit += rhs.capital_in_profit;
        self.capital_in_loss += rhs.capital_in_loss;
    }
}
