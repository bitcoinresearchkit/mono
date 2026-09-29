use std::{collections::BTreeMap, ops::Bound};

use brk_types::{Cents, CentsCompact, Sats};

use super::{Accumulate, UnrealizedState};

#[derive(Debug, Clone)]
pub struct CachedUnrealizedState<S: Accumulate> {
    state: S,
    at_price: CentsCompact,
    cached_output: Option<UnrealizedState>,
}

impl<S: Accumulate> CachedUnrealizedState<S> {
    pub fn compute_fresh(price: Cents, map: &BTreeMap<CentsCompact, Sats>) -> Self {
        let price = price.into();
        let state = Self::compute_raw(price, map);
        Self {
            state,
            at_price: price,
            cached_output: None,
        }
    }

    pub fn current_state(&self) -> UnrealizedState {
        self.state.to_output()
    }

    pub fn get_at_price(
        &mut self,
        new_price: Cents,
        map: &BTreeMap<CentsCompact, Sats>,
    ) -> UnrealizedState {
        let new_price = new_price.into();
        if new_price != self.at_price {
            self.update_for_price_change(new_price, map);
            self.cached_output = None;
        }
        if let Some(output) = &self.cached_output {
            return output.clone();
        }
        self.cached_output.insert(self.state.to_output()).clone()
    }

    pub fn on_receive(&mut self, price: Cents, sats: Sats) {
        self.cached_output = None;
        let price: CentsCompact = price.into();
        let sats_u128 = sats.as_u128();
        let price_u128 = price.as_u128();

        if price <= self.at_price {
            self.state.accumulate_profit(price_u128, sats);
            if price < self.at_price {
                let diff = (self.at_price - price).as_u128();
                *self.state.unrealized_profit() += diff * sats_u128;
            }
        } else {
            self.state.accumulate_loss(price_u128, sats);
            let diff = (price - self.at_price).as_u128();
            *self.state.unrealized_loss() += diff * sats_u128;
        }
    }

    pub fn on_send(&mut self, price: Cents, sats: Sats) {
        self.cached_output = None;
        let price: CentsCompact = price.into();
        let sats_u128 = sats.as_u128();
        let price_u128 = price.as_u128();

        if price <= self.at_price {
            self.state.deaccumulate_profit(price_u128, sats);
            if price < self.at_price {
                let diff = (self.at_price - price).as_u128();
                *self.state.unrealized_profit() -= diff * sats_u128;
            }
        } else {
            self.state.deaccumulate_loss(price_u128, sats);
            let diff = (price - self.at_price).as_u128();
            *self.state.unrealized_loss() -= diff * sats_u128;
        }
    }

    fn update_for_price_change(
        &mut self,
        new_price: CentsCompact,
        map: &BTreeMap<CentsCompact, Sats>,
    ) {
        let old_price = self.at_price;

        if new_price > old_price {
            let delta = (new_price - old_price).as_u128();
            let original_supply_in_profit = self.state.supply_in_profit().as_u128();

            for (&price, &sats) in
                map.range((Bound::Excluded(old_price), Bound::Included(new_price)))
            {
                let sats_u128 = sats.as_u128();
                let price_u128 = price.as_u128();

                self.state.deaccumulate_loss(price_u128, sats);
                self.state.accumulate_profit(price_u128, sats);

                let original_loss = (price - old_price).as_u128();
                *self.state.unrealized_loss() -= original_loss * sats_u128;

                if price < new_price {
                    let new_profit = (new_price - price).as_u128();
                    *self.state.unrealized_profit() += new_profit * sats_u128;
                }
            }

            *self.state.unrealized_profit() += delta * original_supply_in_profit;
            let non_crossing_loss_sats = self.state.supply_in_loss().as_u128();
            *self.state.unrealized_loss() -= delta * non_crossing_loss_sats;
        } else if new_price < old_price {
            let delta = (old_price - new_price).as_u128();
            let original_supply_in_loss = self.state.supply_in_loss().as_u128();

            for (&price, &sats) in
                map.range((Bound::Excluded(new_price), Bound::Included(old_price)))
            {
                let sats_u128 = sats.as_u128();
                let price_u128 = price.as_u128();

                self.state.deaccumulate_profit(price_u128, sats);
                self.state.accumulate_loss(price_u128, sats);

                if price < old_price {
                    let original_profit = (old_price - price).as_u128();
                    *self.state.unrealized_profit() -= original_profit * sats_u128;
                }

                let new_loss = (price - new_price).as_u128();
                *self.state.unrealized_loss() += new_loss * sats_u128;
            }

            *self.state.unrealized_loss() += delta * original_supply_in_loss;
            let non_crossing_profit_sats = self.state.supply_in_profit().as_u128();
            *self.state.unrealized_profit() -= delta * non_crossing_profit_sats;
        }

        self.at_price = new_price;
    }

    fn compute_raw(current_price: CentsCompact, map: &BTreeMap<CentsCompact, Sats>) -> S {
        let mut state = S::default();

        for (&price, &sats) in map {
            let sats_u128 = sats.as_u128();
            let price_u128 = price.as_u128();

            if price <= current_price {
                state.accumulate_profit(price_u128, sats);
                if price < current_price {
                    let diff = (current_price - price).as_u128();
                    *state.unrealized_profit() += diff * sats_u128;
                }
            } else {
                state.accumulate_loss(price_u128, sats);
                let diff = (price - current_price).as_u128();
                *state.unrealized_loss() += diff * sats_u128;
            }
        }

        state
    }
}
