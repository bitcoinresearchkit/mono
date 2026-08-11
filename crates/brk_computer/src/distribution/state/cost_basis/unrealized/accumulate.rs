use brk_types::{Cents, Sats};

use super::{UnrealizedState, WithoutCapital};

/// Accumulates profit and loss across cost-basis entries.
pub trait Accumulate: Default + Clone + Send + Sync + 'static {
    const TRACK_CAPITAL: bool;

    fn to_output(&self) -> UnrealizedState;
    fn core(&self) -> &WithoutCapital;
    fn core_mut(&mut self) -> &mut WithoutCapital;

    fn supply_in_profit(&self) -> Sats {
        self.core().supply_in_profit
    }

    fn supply_in_loss(&self) -> Sats {
        self.core().supply_in_loss
    }

    fn unrealized_profit(&mut self) -> &mut u128 {
        &mut self.core_mut().unrealized_profit
    }

    fn unrealized_loss(&mut self) -> &mut u128 {
        &mut self.core_mut().unrealized_loss
    }

    fn accumulate_profit(&mut self, price: u128, sats: Sats);
    fn accumulate_loss(&mut self, price: u128, sats: Sats);
    fn deaccumulate_profit(&mut self, price: u128, sats: Sats);
    fn deaccumulate_loss(&mut self, price: u128, sats: Sats);
}

#[inline(always)]
pub(super) fn div_btc(raw: u128) -> Cents {
    if raw == 0 {
        Cents::ZERO
    } else {
        Cents::new((raw / Sats::ONE_BTC_U128) as u64)
    }
}
