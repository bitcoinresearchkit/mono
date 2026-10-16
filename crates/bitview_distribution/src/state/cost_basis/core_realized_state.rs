use bitview_primitives::CentsSquaredSats;
use brk_types::{Cents, CentsSats, Sats};
use vecdb::unlikely;

use super::{MinimalRealizedState, RealizedOps};

/// Realized state with activity tracking.
#[derive(Debug, Default, Clone)]
pub struct CoreRealizedState {
    minimal: MinimalRealizedState,
    value_destroyed_raw: u128,
}

impl RealizedOps for CoreRealizedState {
    const TRACK_ACTIVITY: bool = true;

    #[inline]
    fn cap_raw(&self) -> CentsSats {
        self.minimal.cap_raw()
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
            CentsSats::new(self.value_destroyed_raw).to_cents()
        }
    }

    #[inline]
    fn reset_single_iteration_values(&mut self) {
        self.minimal.reset_single_iteration_values();
        self.value_destroyed_raw = 0;
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
    }
}
