use std::cmp::Ordering;

use brk_types::{Cents, CentsSats, CentsSquaredSats, Sats};

use super::RealizedOps;

/// Realized state used by minimal cohorts and address cohorts.
#[derive(Debug, Default, Clone)]
pub struct MinimalRealizedState {
    cap_raw: u128,
    profit_raw: u128,
    loss_raw: u128,
}

impl MinimalRealizedState {
    #[inline]
    pub fn increment_cap(&mut self, cap: CentsSats) {
        self.cap_raw += cap.as_u128();
    }

    #[inline]
    pub fn decrement_cap(&mut self, cap: CentsSats) {
        self.cap_raw -= cap.as_u128();
    }

    #[inline]
    pub fn realize_spend(&mut self, current: CentsSats, previous: CentsSats) {
        match current.cmp(&previous) {
            Ordering::Greater => self.profit_raw += (current - previous).as_u128(),
            Ordering::Less => self.loss_raw += (previous - current).as_u128(),
            Ordering::Equal => {}
        }
        self.decrement_cap(previous);
    }
}

impl RealizedOps for MinimalRealizedState {
    #[inline]
    fn cap_raw(&self) -> CentsSats {
        CentsSats::new(self.cap_raw)
    }

    #[inline]
    fn cap(&self) -> Cents {
        Cents::new((self.cap_raw / Sats::ONE_BTC_U128) as u64)
    }

    #[inline]
    fn profit(&self) -> Cents {
        Cents::new((self.profit_raw / Sats::ONE_BTC_U128) as u64)
    }

    #[inline]
    fn loss(&self) -> Cents {
        Cents::new((self.loss_raw / Sats::ONE_BTC_U128) as u64)
    }

    #[inline]
    fn set_cap_raw(&mut self, cap_raw: CentsSats) {
        self.cap_raw = cap_raw.inner();
    }

    #[inline]
    fn set_capitalized_cap_raw(&mut self, _capitalized_cap_raw: CentsSquaredSats) {}

    #[inline]
    fn reset_single_iteration_values(&mut self) {
        self.profit_raw = 0;
        self.loss_raw = 0;
    }

    #[inline]
    fn increment(&mut self, price: Cents, sats: Sats) {
        if sats.is_not_zero() {
            self.increment_cap(CentsSats::from_price_sats(price, sats));
        }
    }

    #[inline]
    fn increment_snapshot(&mut self, price_sats: CentsSats, _capitalized_cap: CentsSquaredSats) {
        self.increment_cap(price_sats);
    }

    #[inline]
    fn decrement_snapshot(&mut self, price_sats: CentsSats, _capitalized_cap: CentsSquaredSats) {
        self.decrement_cap(price_sats);
    }

    #[inline]
    fn send(
        &mut self,
        _sats: Sats,
        current_ps: CentsSats,
        prev_ps: CentsSats,
        _ath_ps: CentsSats,
        _prev_capitalized_cap: CentsSquaredSats,
    ) {
        self.realize_spend(current_ps, prev_ps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_only_cap_profit_and_loss() {
        let mut state = MinimalRealizedState::default();
        state.increment(Cents::new(10_000), Sats::ONE_BTC);
        state.send(
            Sats::ONE_BTC,
            CentsSats::from_price_sats(Cents::new(15_000), Sats::ONE_BTC),
            CentsSats::from_price_sats(Cents::new(10_000), Sats::ONE_BTC),
            CentsSats::ZERO,
            CentsSquaredSats::ZERO,
        );

        assert_eq!(state.cap(), Cents::ZERO);
        assert_eq!(state.profit(), Cents::new(5_000));
        assert_eq!(state.loss(), Cents::ZERO);
        assert_eq!(state.value_destroyed(), Cents::ZERO);
        assert_eq!(state.sent_in_profit(), Sats::ZERO);
        assert_eq!(state.sent_in_loss(), Sats::ZERO);
    }
}
