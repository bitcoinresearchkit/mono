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
    pub(super) fn to_output(&self) -> UnrealizedState {
        UnrealizedState {
            supply_in_profit: self.supply_in_profit,
            supply_in_loss: self.supply_in_loss,
            unrealized_profit: div_btc(self.unrealized_profit),
            unrealized_loss: div_btc(self.unrealized_loss),
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
