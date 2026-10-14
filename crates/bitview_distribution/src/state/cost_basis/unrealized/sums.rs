use bitview_primitives::CentsCompact;
use brk_types::{Cents, Sats};

use super::UnrealizedState;

/// Supply and unrealized profit/loss sums on each side of a price.
#[derive(Debug, Default, Clone)]
pub(super) struct Sums {
    pub(super) supply_in_profit: Sats,
    pub(super) supply_in_loss: Sats,
    pub(super) unrealized_profit: u128,
    pub(super) unrealized_loss: u128,
}

impl Sums {
    /// The rounded state at `price`, the price these sums were taken at. Each side's capital
    /// is its market value minus its unrealized profit, or plus its unrealized loss, computed
    /// on the exact products and rounded once.
    pub(super) fn to_output(&self, price: CentsCompact) -> UnrealizedState {
        let value = |sats: Sats| price.as_u128() * sats.as_u128();
        let profit_value = value(self.supply_in_profit);
        debug_assert!(profit_value >= self.unrealized_profit);
        UnrealizedState {
            supply_in_profit: self.supply_in_profit,
            supply_in_loss: self.supply_in_loss,
            unrealized_profit: div_btc(self.unrealized_profit),
            unrealized_loss: div_btc(self.unrealized_loss),
            capital_in_profit: div_btc(profit_value - self.unrealized_profit),
            capital_in_loss: div_btc(value(self.supply_in_loss) + self.unrealized_loss),
        }
    }
}

#[inline(always)]
fn div_btc(raw: u128) -> Cents {
    if raw == 0 {
        Cents::ZERO
    } else {
        Cents::new((raw / Sats::ONE_BTC_U128) as u64)
    }
}
