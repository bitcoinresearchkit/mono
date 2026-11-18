use bitview_primitives::CentsCompact;
use brk_types::{Cents, CentsSats, Sats};

use super::UnrealizedState;
use crate::state::CostBasisOps;

/// A cohort's supply on each side of the spot price, with its exact cap (creation price times
/// sats): coins created at or under the spot are in profit, the rest in loss. Coins join and
/// leave the side their price puts them on. The cohort does not know its coins by price: when the
/// spot moves, the one order of every origin by price ([`crate::walk::PriceOrder`]) names the
/// coins it passed, and each changes side through [`Self::cross`]. Unrealized profit and loss
/// derive from the caps, each rounded once.
#[derive(Debug, Default, Clone)]
pub struct SpotSplit {
    profit: Side,
    loss: Side,
    spot: CentsCompact,
}

impl SpotSplit {
    /// Moves sats created at `price` to the other side: the spot passed their price.
    pub(crate) fn cross(&mut self, price: CentsCompact, sats: Sats) {
        let (from, to) = if price <= self.spot {
            (&mut self.profit, &mut self.loss)
        } else {
            (&mut self.loss, &mut self.profit)
        };
        from.sub(price, sats);
        to.add(price, sats);
    }

    /// The split at `spot`, once every coin it passed since the last one has crossed.
    pub(crate) fn at(&mut self, spot: Cents) -> UnrealizedState {
        self.spot = spot.into();
        let value = |side: &Side| self.spot.as_u128() * side.sats.as_u128();
        let cents = |raw: u128| CentsSats::new(raw).to_cents();
        UnrealizedState {
            supply_in_profit: self.profit.sats,
            supply_in_loss: self.loss.sats,
            unrealized_profit: cents(value(&self.profit) - self.profit.cap),
            unrealized_loss: cents(self.loss.cap - value(&self.loss)),
            capital_in_profit: cents(self.profit.cap),
            capital_in_loss: cents(self.loss.cap),
        }
    }

    fn side_mut(&mut self, price: CentsCompact) -> &mut Side {
        if price <= self.spot {
            &mut self.profit
        } else {
            &mut self.loss
        }
    }
}

impl CostBasisOps for SpotSplit {
    #[inline]
    fn increment(&mut self, price: Cents, sats: Sats) {
        let price = price.into();
        self.side_mut(price).add(price, sats);
    }

    #[inline]
    fn decrement(&mut self, price: Cents, sats: Sats) {
        let price = price.into();
        self.side_mut(price).sub(price, sats);
    }
}

#[derive(Debug, Default, Clone)]
struct Side {
    sats: Sats,
    cap: u128,
}

impl Side {
    #[inline]
    fn add(&mut self, price: CentsCompact, sats: Sats) {
        self.sats += sats;
        self.cap += price.as_u128() * sats.as_u128();
    }

    #[inline]
    fn sub(&mut self, price: CentsCompact, sats: Sats) {
        debug_assert!(
            self.sats >= sats,
            "split underflow: coins left the side of the spot they were not on"
        );
        self.sats -= sats;
        self.cap -= price.as_u128() * sats.as_u128();
    }
}
