use std::cmp::Ordering;

use bitview_primitives::CentsSquaredSats;
use brk_types::{Cents, CentsSats, Sats};
use vecdb::unlikely;

use super::{MinimalRealizedState, RealizedOps};

/// Realized state with activity tracking.
#[derive(Debug, Default, Clone)]
pub struct CoreRealizedState {
    minimal: MinimalRealizedState,
    value_destroyed_raw: u128,
    sent_in_profit: Sats,
    sent_in_loss: Sats,
}

impl RealizedOps for CoreRealizedState {
    const TRACK_ACTIVITY: bool = true;

    #[inline]
    fn cap_raw(&self) -> CentsSats {
        self.minimal.cap_raw()
    }

    #[inline]
    fn cap(&self) -> Cents {
        self.minimal.cap()
    }

    #[inline]
    fn profit(&self) -> Cents {
        self.minimal.profit()
    }

    #[inline]
    fn loss(&self) -> Cents {
        self.minimal.loss()
    }

    #[inline]
    fn value_destroyed(&self) -> Cents {
        if unlikely(self.value_destroyed_raw == 0) {
            Cents::ZERO
        } else {
            Cents::new((self.value_destroyed_raw / Sats::ONE_BTC_U128) as u64)
        }
    }

    #[inline]
    fn sent_in_profit(&self) -> Sats {
        self.sent_in_profit
    }

    #[inline]
    fn sent_in_loss(&self) -> Sats {
        self.sent_in_loss
    }

    #[inline]
    fn reset_single_iteration_values(&mut self) {
        self.minimal.reset_single_iteration_values();
        self.value_destroyed_raw = 0;
        self.sent_in_profit = Sats::ZERO;
        self.sent_in_loss = Sats::ZERO;
    }

    #[inline]
    fn increment(&mut self, price: Cents, sats: Sats) {
        self.minimal.increment(price, sats);
    }

    #[inline]
    fn increment_snapshot(&mut self, price_sats: CentsSats, capitalized_cap: CentsSquaredSats) {
        self.minimal.increment_snapshot(price_sats, capitalized_cap);
    }

    #[inline]
    fn decrement_snapshot(&mut self, price_sats: CentsSats, capitalized_cap: CentsSquaredSats) {
        self.minimal.decrement_snapshot(price_sats, capitalized_cap);
    }

    #[inline]
    fn send(
        &mut self,
        sats: Sats,
        current_ps: CentsSats,
        prev_ps: CentsSats,
        ath_ps: CentsSats,
        prev_capitalized_cap: CentsSquaredSats,
    ) {
        self.minimal
            .send(sats, current_ps, prev_ps, ath_ps, prev_capitalized_cap);
        self.value_destroyed_raw += prev_ps.as_u128();
        match current_ps.cmp(&prev_ps) {
            Ordering::Greater | Ordering::Equal => self.sent_in_profit += sats,
            Ordering::Less => self.sent_in_loss += sats,
        }
    }
}

impl CoreRealizedState {
    #[inline(always)]
    pub fn cap_raw_u128(&self) -> u128 {
        self.minimal.cap_raw().as_u128()
    }
}
