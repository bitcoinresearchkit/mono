use std::{collections::BTreeMap, ops::Bound};

use bitview_primitives::CentsCompact;
use brk_types::{Cents, CentsSats, Sats};

use super::UnrealizedState;

/// The supply on each side of a price, with its exact cap (creation price times sats): coins
/// bought at or under the price are in profit, the rest in loss. A price change moves only the
/// coins it crosses; unrealized profit and loss derive from the caps, each rounded once.
#[derive(Debug, Clone)]
pub struct CachedUnrealizedState {
    profit: Side,
    loss: Side,
    at_price: CentsCompact,
}

impl CachedUnrealizedState {
    pub fn compute_fresh(price: Cents, map: &BTreeMap<CentsCompact, Sats>) -> Self {
        let mut this = Self {
            profit: Side::default(),
            loss: Side::default(),
            at_price: price.into(),
        };
        for (&price, &sats) in map {
            this.side_mut(price).add(price, sats);
        }
        this
    }

    pub fn get_at_price(
        &mut self,
        new_price: Cents,
        map: &BTreeMap<CentsCompact, Sats>,
    ) -> UnrealizedState {
        let new_price = new_price.into();
        if new_price != self.at_price {
            self.update_for_price_change(new_price, map);
        }
        self.output()
    }

    pub fn on_receive(&mut self, price: Cents, sats: Sats) {
        let price = price.into();
        self.side_mut(price).add(price, sats);
    }

    pub fn on_send(&mut self, price: Cents, sats: Sats) {
        let price = price.into();
        self.side_mut(price).sub(price, sats);
    }

    fn side_mut(&mut self, price: CentsCompact) -> &mut Side {
        if price <= self.at_price {
            &mut self.profit
        } else {
            &mut self.loss
        }
    }

    fn update_for_price_change(
        &mut self,
        new_price: CentsCompact,
        map: &BTreeMap<CentsCompact, Sats>,
    ) {
        let old_price = self.at_price;
        let (crossed, from, to) = if new_price > old_price {
            (
                (Bound::Excluded(old_price), Bound::Included(new_price)),
                &mut self.loss,
                &mut self.profit,
            )
        } else {
            (
                (Bound::Excluded(new_price), Bound::Included(old_price)),
                &mut self.profit,
                &mut self.loss,
            )
        };
        for (&price, &sats) in map.range(crossed) {
            from.sub(price, sats);
            to.add(price, sats);
        }
        self.at_price = new_price;
    }

    fn output(&self) -> UnrealizedState {
        let value = |side: &Side| self.at_price.as_u128() * side.sats.as_u128();
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
        self.sats -= sats;
        self.cap -= price.as_u128() * sats.as_u128();
    }
}
