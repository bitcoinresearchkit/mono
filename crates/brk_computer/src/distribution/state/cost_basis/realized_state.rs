use brk_types::{Cents, CentsSats, CentsSquaredSats, Sats};

use super::{CoreRealizedState, RealizedOps};

/// Full realized state used by age-range cohorts.
#[derive(Debug, Default, Clone)]
pub struct RealizedState {
    core: CoreRealizedState,
    /// Raw capitalized cap: sum(price squared times sats).
    capitalized_cap_raw: CentsSquaredSats,
    /// Raw realized peak regret: sum((peak minus sell price) times sats).
    peak_regret_raw: u128,
}

impl RealizedOps for RealizedState {
    const TRACK_ACTIVITY: bool = true;

    #[inline]
    fn cap_raw(&self) -> CentsSats {
        self.core.cap_raw()
    }

    #[inline]
    fn cap(&self) -> Cents {
        self.core.cap()
    }

    #[inline]
    fn profit(&self) -> Cents {
        self.core.profit()
    }

    #[inline]
    fn loss(&self) -> Cents {
        self.core.loss()
    }

    #[inline]
    fn value_destroyed(&self) -> Cents {
        self.core.value_destroyed()
    }

    #[inline]
    fn sent_in_profit(&self) -> Sats {
        self.core.sent_in_profit()
    }

    #[inline]
    fn sent_in_loss(&self) -> Sats {
        self.core.sent_in_loss()
    }

    #[inline]
    fn set_cap_raw(&mut self, cap_raw: CentsSats) {
        self.core.set_cap_raw(cap_raw);
    }

    #[inline]
    fn set_capitalized_cap_raw(&mut self, capitalized_cap_raw: CentsSquaredSats) {
        self.capitalized_cap_raw = capitalized_cap_raw;
    }

    #[inline]
    fn reset_single_iteration_values(&mut self) {
        self.core.reset_single_iteration_values();
        self.peak_regret_raw = 0;
    }

    #[inline]
    fn increment(&mut self, price: Cents, sats: Sats) {
        self.core.increment(price, sats);
        if sats.is_not_zero() {
            self.capitalized_cap_raw +=
                CentsSats::from_price_sats(price, sats).to_capitalized_cap(price);
        }
    }

    #[inline]
    fn increment_snapshot(&mut self, price_sats: CentsSats, capitalized_cap: CentsSquaredSats) {
        self.core.increment_snapshot(price_sats, capitalized_cap);
        self.capitalized_cap_raw += capitalized_cap;
    }

    #[inline]
    fn decrement_snapshot(&mut self, price_sats: CentsSats, capitalized_cap: CentsSquaredSats) {
        self.core.decrement_snapshot(price_sats, capitalized_cap);
        self.capitalized_cap_raw -= capitalized_cap;
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
        self.core
            .send(sats, current_ps, prev_ps, ath_ps, prev_capitalized_cap);
        self.peak_regret_raw += (ath_ps - current_ps).as_u128();
        self.capitalized_cap_raw -= prev_capitalized_cap;
    }
}

impl RealizedState {
    #[inline]
    pub(crate) fn cap_raw(&self) -> CentsSats {
        CentsSats::new(self.core.cap_raw_u128())
    }

    #[inline]
    pub(crate) fn capitalized_cap_raw(&self) -> CentsSquaredSats {
        self.capitalized_cap_raw
    }

    #[inline]
    pub(crate) fn peak_regret_raw(&self) -> u128 {
        self.peak_regret_raw
    }
}
