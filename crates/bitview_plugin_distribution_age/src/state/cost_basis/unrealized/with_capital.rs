use brk_types::Sats;

use super::{Accumulate, UnrealizedState, WithoutCapital};

/// Unrealized cache state with capitalized-cap tracking.
#[derive(Debug, Default, Clone)]
pub struct WithCapital {
    core: WithoutCapital,
    capitalized_cap_in_profit: u128,
    capitalized_cap_in_loss: u128,
}

impl Accumulate for WithCapital {
    const TRACK_CAPITAL: bool = true;

    fn to_output(&self) -> UnrealizedState {
        UnrealizedState {
            capitalized_cap_in_profit_raw: self.capitalized_cap_in_profit,
            capitalized_cap_in_loss_raw: self.capitalized_cap_in_loss,
            ..Accumulate::to_output(&self.core)
        }
    }

    fn core(&self) -> &WithoutCapital {
        &self.core
    }

    fn core_mut(&mut self) -> &mut WithoutCapital {
        &mut self.core
    }

    #[inline(always)]
    fn accumulate_profit(&mut self, price: u128, sats: Sats) {
        self.core.supply_in_profit += sats;
        let invested = price * sats.as_u128();
        self.capitalized_cap_in_profit += price * invested;
    }

    #[inline(always)]
    fn accumulate_loss(&mut self, price: u128, sats: Sats) {
        self.core.supply_in_loss += sats;
        let invested = price * sats.as_u128();
        self.capitalized_cap_in_loss += price * invested;
    }

    #[inline(always)]
    fn deaccumulate_profit(&mut self, price: u128, sats: Sats) {
        self.core.supply_in_profit -= sats;
        let invested = price * sats.as_u128();
        self.capitalized_cap_in_profit -= price * invested;
    }

    #[inline(always)]
    fn deaccumulate_loss(&mut self, price: u128, sats: Sats) {
        self.core.supply_in_loss -= sats;
        let invested = price * sats.as_u128();
        self.capitalized_cap_in_loss -= price * invested;
    }
}
