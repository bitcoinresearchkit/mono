use brk_types::Sats;

use super::{Accumulate, UnrealizedState, accumulate::div_btc};

/// Supply and unrealized profit/loss cache state without capital tracking.
#[derive(Debug, Default, Clone)]
pub struct WithoutCapital {
    pub supply_in_profit: Sats,
    pub supply_in_loss: Sats,
    pub unrealized_profit: u128,
    pub unrealized_loss: u128,
}

impl Accumulate for WithoutCapital {
    const TRACK_CAPITAL: bool = false;

    fn to_output(&self) -> UnrealizedState {
        UnrealizedState {
            supply_in_profit: self.supply_in_profit,
            supply_in_loss: self.supply_in_loss,
            unrealized_profit: div_btc(self.unrealized_profit),
            unrealized_loss: div_btc(self.unrealized_loss),
            ..UnrealizedState::ZERO
        }
    }

    fn core(&self) -> &WithoutCapital {
        self
    }

    fn core_mut(&mut self) -> &mut WithoutCapital {
        self
    }

    #[inline(always)]
    fn accumulate_profit(&mut self, _price: u128, sats: Sats) {
        self.supply_in_profit += sats;
    }

    #[inline(always)]
    fn accumulate_loss(&mut self, _price: u128, sats: Sats) {
        self.supply_in_loss += sats;
    }

    #[inline(always)]
    fn deaccumulate_profit(&mut self, _price: u128, sats: Sats) {
        self.supply_in_profit -= sats;
    }

    #[inline(always)]
    fn deaccumulate_loss(&mut self, _price: u128, sats: Sats) {
        self.supply_in_loss -= sats;
    }
}
