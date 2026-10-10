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
    /// The capital of the supply in profit and of the supply in loss at `price`: each side's
    /// market value minus its unrealized profit, or plus its unrealized loss.
    pub fn capital_split(&self, price: Cents) -> (Cents, Cents) {
        let value = |sats: Sats| sats.as_u128() * price.as_u128() / Sats::ONE_BTC_U128;
        let in_profit =
            value(self.supply_in_profit).saturating_sub(self.unrealized_profit.as_u128());
        let in_loss = value(self.supply_in_loss) + self.unrealized_loss.as_u128();
        (Cents::new(in_profit as u64), Cents::new(in_loss as u64))
    }

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
